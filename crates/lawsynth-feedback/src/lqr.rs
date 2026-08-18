//! Infinite-horizon LQR via a deterministic Kleinman (Newton–Riccati) iteration.
//!
//! The continuous-time algebraic Riccati equation (CARE)
//! `AᵀP + PA − PBR⁻¹BᵀP + Q = 0` is solved by Kleinman's iteration: starting
//! from a stabilizing gain `K₀`, each step solves the Lyapunov equation
//! `(A − BKᵢ)ᵀ P + P (A − BKᵢ) = −(Q + KᵢᵀRKᵢ)` and updates `K = R⁻¹BᵀP` until
//! the gain stops moving. The initial `K₀` is built by Bass's algorithm, which
//! is deterministic and needs no eigenvalue placement. The optimal control law
//! is `u = −K x`.

use lawsynth_koopman::{Matrix, eigen};

use crate::error::FeedbackError;
use crate::gain::Gain;
use crate::linalg;

/// Maximum Kleinman iterations before declaring non-convergence.
const MAX_ITERATIONS: usize = 200;
/// Relative tolerance on successive gains for convergence.
const CONVERGENCE_TOLERANCE: f64 = 1e-13;

/// Solves the infinite-horizon LQR problem for `ẋ = A x + B u` with state weight
/// `Q ⪰ 0` and control weight `R ≻ 0`, returning the gain `K` (law `u = −K x`),
/// the achieved closed-loop poles, and the solved Riccati matrix `P`.
///
/// # Errors
/// - [`FeedbackError::ShapeMismatch`] for inconsistent operand shapes.
/// - [`FeedbackError::NotSymmetric`] if `Q` or `R` is not symmetric.
/// - [`FeedbackError::NotPositiveSemidefinite`] if `Q` has a negative eigenvalue.
/// - [`FeedbackError::NotPositiveDefinite`] if `R` is not positive definite.
/// - [`FeedbackError::NotStabilizable`] if no initial stabilizing gain exists.
/// - [`FeedbackError::NoConvergence`] if the iteration exhausts its budget.
pub fn lqr(a: &Matrix, b: &Matrix, q: &Matrix, r: &Matrix) -> Result<Gain, FeedbackError> {
    let n = a.rows();
    if a.cols() != n {
        return Err(FeedbackError::NonSquare);
    }
    if n == 0 {
        return Err(FeedbackError::EmptyMatrix);
    }
    let m = b.cols();
    if b.rows() != n {
        return Err(FeedbackError::ShapeMismatch);
    }
    if q.rows() != n || q.cols() != n || r.rows() != m || r.cols() != m {
        return Err(FeedbackError::ShapeMismatch);
    }
    if !linalg::is_finite(a)
        || !linalg::is_finite(b)
        || !linalg::is_finite(q)
        || !linalg::is_finite(r)
    {
        return Err(FeedbackError::NonFiniteValue);
    }

    validate_weights(q, r)?;
    let r_inverse = linalg::invert(r).map_err(|_| FeedbackError::NotPositiveDefinite)?;

    // Initial stabilizing gain (Bass), then Kleinman iteration to the CARE root.
    let mut k = bass_initial_gain(a, b)?;
    let mut converged = false;
    for _ in 0..MAX_ITERATIONS {
        let p = riccati_step_solution(a, b, q, r, &k)?;
        let k_next = linalg::mm(&linalg::mm(&r_inverse, &b.transpose())?, &p)?;
        let movement = linalg::max_abs_diff(&k_next, &k);
        let scale = 1.0 + linalg::max_abs(&k_next);
        k = k_next;
        if movement <= CONVERGENCE_TOLERANCE * scale {
            converged = true;
            break;
        }
    }
    if !converged {
        return Err(FeedbackError::NoConvergence);
    }

    // Final `P` consistent with the converged gain, plus the achieved spectrum.
    let p = riccati_step_solution(a, b, q, r, &k)?;
    let closed_loop = linalg::sub(a, &linalg::mm(b, &k)?);
    let achieved_poles = eigen(&closed_loop)?.values;

    Ok(Gain { k, achieved_poles, p: Some(p) })
}

/// Validates that `Q` is symmetric PSD and `R` is symmetric positive definite.
fn validate_weights(q: &Matrix, r: &Matrix) -> Result<(), FeedbackError> {
    let q_tolerance = 1e-9 * (1.0 + linalg::max_abs(q));
    if !linalg::is_symmetric(q, q_tolerance) {
        return Err(FeedbackError::NotSymmetric);
    }
    let r_tolerance = 1e-9 * (1.0 + linalg::max_abs(r));
    if !linalg::is_symmetric(r, r_tolerance) {
        return Err(FeedbackError::NotSymmetric);
    }

    // Symmetric matrices have real eigenvalues; check the smallest.
    let q_min = min_eigenvalue_real(q)?;
    if q_min < -1e-9 * (1.0 + linalg::frobenius_norm(q)) {
        return Err(FeedbackError::NotPositiveSemidefinite);
    }
    let r_min = min_eigenvalue_real(r)?;
    if r_min <= 1e-12 * (1.0 + linalg::frobenius_norm(r)) {
        return Err(FeedbackError::NotPositiveDefinite);
    }
    Ok(())
}

/// The smallest real part among a matrix's eigenvalues (real for symmetric input).
fn min_eigenvalue_real(a: &Matrix) -> Result<f64, FeedbackError> {
    let decomposition = eigen(a)?;
    Ok(decomposition.values.iter().map(|value| value.re).fold(f64::INFINITY, f64::min))
}

/// One Kleinman step: solve `Acᵀ P + P Ac = −(Q + KᵀRK)` for the current gain.
fn riccati_step_solution(
    a: &Matrix,
    b: &Matrix,
    q: &Matrix,
    r: &Matrix,
    k: &Matrix,
) -> Result<Matrix, FeedbackError> {
    let closed_loop = linalg::sub(a, &linalg::mm(b, k)?);
    let ktr = linalg::mm(&k.transpose(), r)?;
    let ktrk = linalg::mm(&ktr, k)?;
    let weight = linalg::add(q, &ktrk);
    let solution = linalg::lyapunov(&closed_loop.transpose(), &linalg::scale(&weight, -1.0))?;
    Ok(linalg::symmetrize(&solution))
}

/// Bass's algorithm for an initial stabilizing gain `K₀` (so `A − BK₀` is
/// Hurwitz). With `β > ‖A‖`, `A + βI` is anti-stable, so the Lyapunov solution
/// of `(A + βI) Z + Z (A + βI)ᵀ = 2 B Bᵀ` is positive definite for a
/// controllable pair, and `K₀ = Bᵀ Z⁻¹` stabilizes the loop.
fn bass_initial_gain(a: &Matrix, b: &Matrix) -> Result<Matrix, FeedbackError> {
    let n = a.rows();
    let beta = linalg::frobenius_norm(a) + 1.0;
    let mut shifted = a.clone();
    for i in 0..n {
        shifted.set(i, i, shifted.get(i, i) + beta);
    }
    let bbt = linalg::mm(b, &b.transpose())?;
    let z = linalg::lyapunov(&shifted, &linalg::scale(&bbt, 2.0))?;
    let z_inverse = linalg::invert(&z).map_err(|_| FeedbackError::NotStabilizable)?;
    linalg::mm(&b.transpose(), &z_inverse)
}
