//! Luenberger observer and steady-state Kalman-filter design by duality.
//!
//! A state estimator reconstructs the full state `x` of `ẋ = A x + B u`,
//! `y = C x` from the partial measurement `y` by running a copy of the model
//! corrected by the innovation `y − C x̂`:
//!
//! ```text
//! x̂̇ = A x̂ + B u + L (y − C x̂),   so the error e = x − x̂ obeys ė = (A − L C) e.
//! ```
//!
//! Choosing the gain `L` to shape `A − L C` is the **exact dual** of choosing a
//! feedback gain `K` to shape `A − B K`, because `(A − L C)ᵀ = Aᵀ − Cᵀ Lᵀ`. So
//! we reuse `lawsynth-feedback`:
//!
//! - **Pole placement** ([`design_observer`]) places the error poles by calling
//!   Ackermann's formula on the dual pair `(Aᵀ, Cᵀ)` and transposing the gain:
//!   `L = place_poles(Aᵀ, Cᵀ, desired)ᵀ`. Single output ⇒ `Cᵀ` is `n × 1`, the
//!   SISO case Ackermann handles. Requires observability (dual of
//!   controllability).
//! - **Kalman filter** ([`kalman_filter`]) is the dual of LQR. The feedback CARE
//!   `AᵀP + PA − PBR⁻¹BᵀP + Q = 0` becomes, under `(A, B) → (Aᵀ, Cᵀ)`, the
//!   **filter** CARE `A P + P Aᵀ − P Cᵀ R⁻¹ C P + Q = 0`. Hence
//!   `kalman_filter(A, C, Q, R)` calls `lqr(Aᵀ, Cᵀ, Q, R)`: the returned Riccati
//!   matrix is the error covariance `P`, and the optimal gain is
//!   `L = (lqr gain)ᵀ = P Cᵀ R⁻¹`. Requires detectability and `R ≻ 0`.

use lawsynth_feedback::{lqr, place_poles};
use lawsynth_koopman::{Complex, Matrix, eigen};

use crate::error::{EstimateError, from_feedback};
use crate::linalg;

/// How an observer gain was designed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverMethod {
    /// Error poles placed exactly by Ackermann's formula on the dual pair.
    PolePlacement,
    /// Steady-state Kalman gain from the dual continuous-time filter CARE.
    Kalman,
}

/// A designed state estimator: the gain, the achieved error spectrum, and — for
/// the Kalman filter — the steady-state error covariance.
///
/// The estimator dynamics are `x̂̇ = A x̂ + B u + L (y − C x̂)`, so the
/// estimation error `e = x − x̂` evolves as `ė = (A − L C) e`. The
/// [`error_poles`](Observer::error_poles) are the eigenvalues of `A − L C`
/// computed with the shared deterministic eigensolver, so a caller can confirm
/// the estimator converges (error decays) without re-deriving anything.
#[derive(Clone, Debug)]
pub struct Observer {
    /// The observer gain `L`, shaped `n × p` (`n` states, `p` outputs).
    pub gain: Matrix,
    /// Eigenvalues of the error dynamics `A − L C`, in the eigensolver's
    /// canonical order (descending modulus, deterministic tie-breaks).
    pub error_poles: Vec<Complex>,
    /// The steady-state error covariance `P` (Kalman only); `None` for pole
    /// placement, which forms no covariance.
    pub covariance: Option<Matrix>,
    /// How this gain was designed.
    pub method: ObserverMethod,
}

impl Observer {
    /// The number of states `n` (rows of `L`).
    pub fn states(&self) -> usize {
        self.gain.rows()
    }

    /// The number of measured outputs `p` (columns of `L`).
    pub fn outputs(&self) -> usize {
        self.gain.cols()
    }

    /// True when every error pole has strictly negative real part, up to a small
    /// absolute margin (the error dynamics `A − L C` are Hurwitz, so `x̂ → x`).
    pub fn is_convergent(&self, margin: f64) -> bool {
        self.error_poles.iter().all(|pole| pole.re < -margin)
    }
}

/// Builds the observability matrix `O = [C; C A; …; C Aⁿ⁻¹]`, shaped `np × n`.
///
/// The pair `(A, C)` is observable iff `O` has full column rank `n`.
pub fn observability_matrix(a: &Matrix, c: &Matrix) -> Result<Matrix, EstimateError> {
    let n = a.rows();
    if a.cols() != n {
        return Err(EstimateError::NonSquare);
    }
    if n == 0 {
        return Err(EstimateError::EmptyMatrix);
    }
    let p = c.rows();
    if c.cols() != n {
        return Err(EstimateError::ShapeMismatch);
    }
    let mut o = Matrix::zeros(n * p, n);
    let mut block = c.clone(); // C Aᵏ, starting at k = 0.
    for k in 0..n {
        for i in 0..p {
            for j in 0..n {
                o.set(k * p + i, j, block.get(i, j));
            }
        }
        if k + 1 < n {
            block = linalg::mm(&block, a)?;
        }
    }
    Ok(o)
}

/// True when `(A, C)` is observable: the observability matrix has full column
/// rank `n`.
pub fn is_observable(a: &Matrix, c: &Matrix) -> Result<bool, EstimateError> {
    let o = observability_matrix(a, c)?;
    let tol = 1e-9 * (1.0 + linalg::max_abs(&o));
    Ok(linalg::column_rank(&o, tol) == a.rows())
}

/// Designs a Luenberger observer placing the error poles of `A − L C` at
/// `desired`, by pole placement on the dual pair `(Aᵀ, Cᵀ)`.
///
/// `C` must be `1 × n` (a single measured output), the SISO case Ackermann's
/// formula solves; a taller `C` is rejected with [`EstimateError::MultiOutput`].
///
/// # Errors
/// - [`EstimateError::NonSquare`] if `A` is not square.
/// - [`EstimateError::ShapeMismatch`] if `C` has the wrong number of columns.
/// - [`EstimateError::MultiOutput`] if `C` is not `1 × n`.
/// - [`EstimateError::PoleCountMismatch`] if `desired.len() != n`.
/// - [`EstimateError::NonRealDesignPoles`] if the poles are not conjugate-closed.
/// - [`EstimateError::Unobservable`] if `(A, C)` is not observable.
pub fn design_observer(
    a: &Matrix,
    c: &Matrix,
    desired: &[Complex],
) -> Result<Observer, EstimateError> {
    let n = a.rows();
    if a.cols() != n {
        return Err(EstimateError::NonSquare);
    }
    if n == 0 {
        return Err(EstimateError::EmptyMatrix);
    }
    if c.cols() != n {
        return Err(EstimateError::ShapeMismatch);
    }
    if c.rows() != 1 {
        return Err(EstimateError::MultiOutput);
    }
    if !linalg::is_finite(a) || !linalg::is_finite(c) {
        return Err(EstimateError::NonFiniteValue);
    }
    if desired.len() != n {
        return Err(EstimateError::PoleCountMismatch);
    }

    // Observability is required, not assumed — check before placing.
    if !is_observable(a, c)? {
        return Err(EstimateError::Unobservable);
    }

    // Dual pole placement: place poles of (Aᵀ − Cᵀ Kᵀ) = (A − L C)ᵀ, then
    // transpose. `Cᵀ` is n×1, the single-input pair Ackermann needs.
    let a_dual = a.transpose();
    let c_dual = c.transpose();
    let gain_dual = place_poles(&a_dual, &c_dual, desired).map_err(from_feedback)?;
    let l = gain_dual.k.transpose(); // (1×n)ᵀ = n×1.

    // Verify the realized error spectrum via the shared eigensolver.
    let error_dynamics = linalg::error_dynamics(a, &l, c)?;
    let error_poles = eigen(&error_dynamics)?.values;

    Ok(Observer { gain: l, error_poles, covariance: None, method: ObserverMethod::PolePlacement })
}

/// Designs a steady-state continuous-time Kalman filter for `ẋ = A x + B u`,
/// `y = C x` with process covariance `Q ⪰ 0` and measurement covariance
/// `R ≻ 0`, by solving the dual filter CARE via [`lqr`] on `(Aᵀ, Cᵀ)`.
///
/// Returns the optimal gain `L = P Cᵀ R⁻¹` and the steady-state error
/// covariance `P`, the symmetric positive-(semi)definite solution of
/// `A P + P Aᵀ − P Cᵀ R⁻¹ C P + Q = 0`. Unlike [`design_observer`] this handles
/// multiple outputs (`p ≥ 1`), since the underlying LQR is multi-input.
///
/// # Errors
/// - [`EstimateError::NonSquare`] / [`EstimateError::ShapeMismatch`] for bad shapes.
/// - [`EstimateError::NotSymmetric`] if `Q` or `R` is not symmetric.
/// - [`EstimateError::NotPositiveSemidefinite`] if `Q` has a negative eigenvalue.
/// - [`EstimateError::NotPositiveDefinite`] if `R` is not positive definite.
/// - [`EstimateError::NotDetectable`] if `(A, C)` is not detectable.
/// - [`EstimateError::NoConvergence`] if the dual Riccati iteration fails.
pub fn kalman_filter(
    a: &Matrix,
    c: &Matrix,
    process_cov: &Matrix,
    measurement_cov: &Matrix,
) -> Result<Observer, EstimateError> {
    let n = a.rows();
    if a.cols() != n {
        return Err(EstimateError::NonSquare);
    }
    if n == 0 {
        return Err(EstimateError::EmptyMatrix);
    }
    let p = c.rows();
    if c.cols() != n {
        return Err(EstimateError::ShapeMismatch);
    }
    if process_cov.rows() != n || process_cov.cols() != n {
        return Err(EstimateError::ShapeMismatch);
    }
    if measurement_cov.rows() != p || measurement_cov.cols() != p {
        return Err(EstimateError::ShapeMismatch);
    }
    if !linalg::is_finite(a) || !linalg::is_finite(c) {
        return Err(EstimateError::NonFiniteValue);
    }

    // Dual LQR: lqr(Aᵀ, Cᵀ, Q, R) solves A P + P Aᵀ − P Cᵀ R⁻¹ C P + Q = 0 for
    // X = P and yields gain R⁻¹ C P; the Kalman gain is its transpose.
    let a_dual = a.transpose();
    let c_dual = c.transpose();
    let dual = lqr(&a_dual, &c_dual, process_cov, measurement_cov).map_err(from_feedback)?;
    let l = dual.k.transpose(); // (R⁻¹ C P)ᵀ = P Cᵀ R⁻¹, shaped n×p.
    let covariance = dual.p; // the solved filter Riccati matrix P.

    let error_dynamics = linalg::error_dynamics(a, &l, c)?;
    let error_poles = eigen(&error_dynamics)?.values;

    Ok(Observer { gain: l, error_poles, covariance, method: ObserverMethod::Kalman })
}
