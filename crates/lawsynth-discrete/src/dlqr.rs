//! Infinite-horizon discrete LQR via the discrete algebraic Riccati equation.

use lawsynth_koopman::{Matrix, eigen};

use crate::dare;
use crate::error::DiscreteError;
use crate::gain::DiscreteGain;
use crate::linalg;
use crate::validate;

/// Solves the infinite-horizon discrete LQR problem for
/// `x_{k+1} = A x_k + B u_k` with state weight `Q ⪰ 0` and control weight
/// `R ≻ 0`, minimizing `Σ (xₖᵀQxₖ + uₖᵀRuₖ)`.
///
/// Returns the gain `K` (law `u = −K x`), the achieved closed-loop poles of
/// `A − BK` (verified inside the unit circle), and the solved DARE matrix `P`.
///
/// # Errors
/// - [`DiscreteError::NonSquare`] if `A` is not square.
/// - [`DiscreteError::EmptyMatrix`] if `A` has order zero.
/// - [`DiscreteError::ShapeMismatch`] for inconsistent operand shapes.
/// - [`DiscreteError::NonFiniteValue`] for a non-finite entry.
/// - [`DiscreteError::NotSymmetric`] if `Q` or `R` is not symmetric.
/// - [`DiscreteError::NotPositiveSemidefinite`] if `Q` has a negative eigenvalue.
/// - [`DiscreteError::NotPositiveDefinite`] if `R` is not positive definite.
/// - [`DiscreteError::NotConvergent`] if the DARE iteration diverges or fails to
///   settle (the pair is not stabilizable/detectable).
pub fn dlqr(a: &Matrix, b: &Matrix, q: &Matrix, r: &Matrix) -> Result<DiscreteGain, DiscreteError> {
    let n = a.rows();
    if a.cols() != n {
        return Err(DiscreteError::NonSquare);
    }
    if n == 0 {
        return Err(DiscreteError::EmptyMatrix);
    }
    let m = b.cols();
    if b.rows() != n {
        return Err(DiscreteError::ShapeMismatch);
    }
    if q.rows() != n || q.cols() != n || r.rows() != m || r.cols() != m {
        return Err(DiscreteError::ShapeMismatch);
    }
    if !linalg::is_finite(a)
        || !linalg::is_finite(b)
        || !linalg::is_finite(q)
        || !linalg::is_finite(r)
    {
        return Err(DiscreteError::NonFiniteValue);
    }

    validate::symmetric_psd(q)?;
    validate::symmetric_pd(r)?;

    let p = dare::solve_dare(a, b, q, r)?;
    let k = dare::gain_from_solution(a, b, r, &p)?;

    // Achieved discrete spectrum of the closed loop A − BK.
    let closed_loop = linalg::sub(a, &linalg::mm(b, &k)?);
    let achieved_poles = eigen(&closed_loop)?.values;

    Ok(DiscreteGain { k, achieved_poles, p })
}
