//! The steady-state discrete Kalman filter via the dual (filter) DARE.
//!
//! Estimator design is the exact dual of feedback design: the filter DARE for
//! `(A, C)` is the control DARE for `(Aᵀ, Cᵀ)`. This routine therefore reuses
//! the same [`crate::dare`] value iteration on the transposed pair, then reads
//! the *predictor* (a-priori) gain off the solution.

use lawsynth_koopman::{Matrix, eigen};

use crate::dare;
use crate::error::DiscreteError;
use crate::linalg;
use crate::observer::{DiscreteObserver, ObserverMethod};
use crate::validate;

/// Designs the steady-state discrete Kalman filter for `x_{k+1} = A x_k + w_k`,
/// `y_k = C x_k + v_k` with process covariance `Q ⪰ 0` and measurement
/// covariance `R ≻ 0`.
///
/// Solves the filter DARE `P = APAᵀ − APCᵀ(R + CPCᵀ)⁻¹CPAᵀ + Q` and returns the
/// **predictor** gain `L = APCᵀ(R + CPCᵀ)⁻¹`, the error covariance `P`, and the
/// achieved error poles of `A − LC` (verified inside the unit circle). The
/// predictor form propagates `x̂_{k+1} = A x̂_k + B u_k + L(y_k − C x̂_k)`, whose
/// error obeys `e_{k+1} = (A − LC) e_k`.
///
/// Multiple outputs (`p ≥ 1`) are supported, since the dual control problem is
/// multi-input.
///
/// # Errors
/// Mirrors [`crate::dlqr`] on the dual pair: shape/finiteness errors,
/// [`DiscreteError::NotSymmetric`] / [`DiscreteError::NotPositiveSemidefinite`] /
/// [`DiscreteError::NotPositiveDefinite`] for bad covariances, and
/// [`DiscreteError::NotConvergent`] when `(A, C)` is not detectable.
pub fn discrete_kalman(
    a: &Matrix,
    c: &Matrix,
    q: &Matrix,
    r: &Matrix,
) -> Result<DiscreteObserver, DiscreteError> {
    let n = a.rows();
    if a.cols() != n {
        return Err(DiscreteError::NonSquare);
    }
    if n == 0 {
        return Err(DiscreteError::EmptyMatrix);
    }
    let p_outputs = c.rows();
    if c.cols() != n {
        return Err(DiscreteError::ShapeMismatch);
    }
    if q.rows() != n || q.cols() != n || r.rows() != p_outputs || r.cols() != p_outputs {
        return Err(DiscreteError::ShapeMismatch);
    }
    if !linalg::is_finite(a)
        || !linalg::is_finite(c)
        || !linalg::is_finite(q)
        || !linalg::is_finite(r)
    {
        return Err(DiscreteError::NonFiniteValue);
    }

    validate::symmetric_psd(q)?;
    validate::symmetric_pd(r)?;

    // Dual pair (Aᵀ, Cᵀ): the control DARE for it is the filter DARE for (A, C).
    let at = a.transpose();
    let ct = c.transpose();
    let p = dare::solve_dare(&at, &ct, q, r)?;

    // Predictor gain L = A P Cᵀ (R + C P Cᵀ)⁻¹.
    let apct = linalg::mm3(a, &p, &ct)?; // n × p
    let cpct = linalg::mm3(c, &p, &ct)?; // p × p
    let inner_inv = linalg::invert(&linalg::add(r, &cpct))?;
    let l = linalg::mm(&apct, &inner_inv)?;

    // Achieved discrete error spectrum of A − LC.
    let error_dynamics = linalg::sub(a, &linalg::mm(&l, c)?);
    let error_poles = eigen(&error_dynamics)?.values;

    Ok(DiscreteObserver { l, error_poles, p: Some(p), method: ObserverMethod::Kalman })
}
