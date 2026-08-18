//! The discrete algebraic Riccati equation (DARE) solved by a deterministic
//! value (fixed-point) iteration.
//!
//! For the control DARE
//!
//! ```text
//! P = AᵀPA − AᵀPB (R + BᵀPB)⁻¹ BᵀPA + Q,
//! ```
//!
//! the map `P ↦ AᵀPA − AᵀPB(R+BᵀPB)⁻¹BᵀPA + Q` is iterated from `P₀ = Q` to a
//! fixed convergence tolerance. For a stabilizable pair with a detectable
//! `(A, Q^{1/2})` this iteration converges (linearly) to the unique symmetric
//! stabilizing solution; the corresponding gain `K = (R + BᵀPB)⁻¹BᵀPA` places
//! the closed-loop spectrum of `A − BK` inside the unit circle.
//!
//! The Kalman filter reuses this exact solver on the dual pair `(Aᵀ, Cᵀ)`, so
//! the whole crate shares one audited Riccati routine.

use lawsynth_koopman::Matrix;

use crate::error::DiscreteError;
use crate::linalg;

/// Maximum value iterations before declaring non-convergence.
const MAX_ITERATIONS: usize = 100_000;
/// Relative tolerance on successive iterates for convergence.
const CONVERGENCE_TOLERANCE: f64 = 1e-13;

/// Solves the control DARE `P = AᵀPA − AᵀPB(R+BᵀPB)⁻¹BᵀPA + Q` for the symmetric
/// stabilizing `P`, by value iteration from `P₀ = Q`.
///
/// `a` is `n × n`, `b` is `n × m`, `q` is `n × n`, `r` is `m × m`. Shapes and
/// finiteness are validated by the callers; this routine assumes them and
/// focuses on the iteration. Returns [`DiscreteError::NotConvergent`] if the
/// iterate diverges or fails to settle within [`MAX_ITERATIONS`], and
/// [`DiscreteError::SingularSystem`] only if `R + BᵀPB` becomes singular (it is
/// positive definite for `R ≻ 0`, so this indicates a numerical breakdown).
pub fn solve_dare(a: &Matrix, b: &Matrix, q: &Matrix, r: &Matrix) -> Result<Matrix, DiscreteError> {
    let at = a.transpose();
    let bt = b.transpose();
    // A generous divergence guard scaled by the problem data. An unstabilizable
    // unstable mode drives the iterate to grow without bound; catch it early.
    let divergence_bound =
        1e14 * (1.0 + linalg::max_abs(q) + linalg::max_abs(a) + linalg::max_abs(b));

    let mut p = q.clone();
    let mut converged = false;
    for _ in 0..MAX_ITERATIONS {
        let next = dare_step(a, b, q, r, &at, &bt, &p)?;
        if !linalg::is_finite(&next) || linalg::max_abs(&next) > divergence_bound {
            return Err(DiscreteError::NotConvergent);
        }
        let movement = linalg::max_abs_diff(&next, &p);
        let scale = 1.0 + linalg::max_abs(&next);
        p = next;
        if movement <= CONVERGENCE_TOLERANCE * scale {
            converged = true;
            break;
        }
    }
    if !converged {
        return Err(DiscreteError::NotConvergent);
    }
    Ok(p)
}

/// One value-iteration step of the control DARE, returned symmetrized.
fn dare_step(
    a: &Matrix,
    b: &Matrix,
    q: &Matrix,
    r: &Matrix,
    at: &Matrix,
    bt: &Matrix,
    p: &Matrix,
) -> Result<Matrix, DiscreteError> {
    let btp = linalg::mm(bt, p)?; // Bᵀ P            (m × n)
    let btpb = linalg::mm(&btp, b)?; // Bᵀ P B       (m × m)
    let inner = linalg::add(r, &btpb); // R + Bᵀ P B
    let inner_inv = linalg::invert(&inner)?;
    let btpa = linalg::mm(&btp, a)?; // Bᵀ P A        (m × n)
    let atpb = btpa.transpose(); // Aᵀ P B            (n × m)
    let atpa = linalg::mm3(at, p, a)?; // Aᵀ P A      (n × n)
    let correction = linalg::mm3(&atpb, &inner_inv, &btpa)?;
    Ok(linalg::symmetrize(&linalg::add(&linalg::sub(&atpa, &correction), q)))
}

/// The optimal gain `K = (R + BᵀPB)⁻¹ BᵀPA` for the solved `P`.
pub fn gain_from_solution(
    a: &Matrix,
    b: &Matrix,
    r: &Matrix,
    p: &Matrix,
) -> Result<Matrix, DiscreteError> {
    let bt = b.transpose();
    let btp = linalg::mm(&bt, p)?;
    let inner = linalg::add(r, &linalg::mm(&btp, b)?);
    let inner_inv = linalg::invert(&inner)?;
    let btpa = linalg::mm(&btp, a)?;
    linalg::mm(&inner_inv, &btpa)
}
