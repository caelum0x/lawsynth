//! Boundary validation of the symmetric weight/covariance matrices, shared by
//! the DLQR and Kalman designs.
//!
//! Symmetry is checked directly; definiteness is checked from the shared
//! deterministic eigensolver (symmetric matrices have real eigenvalues, so the
//! smallest one certifies (semi)definiteness).

use lawsynth_koopman::{Matrix, eigen};

use crate::error::DiscreteError;
use crate::linalg;

/// Verifies `q` is symmetric positive semidefinite (`Q ⪰ 0`).
pub fn symmetric_psd(q: &Matrix) -> Result<(), DiscreteError> {
    let tolerance = 1e-9 * (1.0 + linalg::max_abs(q));
    if !linalg::is_symmetric(q, tolerance) {
        return Err(DiscreteError::NotSymmetric);
    }
    let min = min_eigenvalue(q)?;
    if min < -1e-9 * (1.0 + linalg::frobenius_norm(q)) {
        return Err(DiscreteError::NotPositiveSemidefinite);
    }
    Ok(())
}

/// Verifies `r` is symmetric positive definite (`R ≻ 0`, hence invertible).
pub fn symmetric_pd(r: &Matrix) -> Result<(), DiscreteError> {
    let tolerance = 1e-9 * (1.0 + linalg::max_abs(r));
    if !linalg::is_symmetric(r, tolerance) {
        return Err(DiscreteError::NotSymmetric);
    }
    let min = min_eigenvalue(r)?;
    if min <= 1e-12 * (1.0 + linalg::frobenius_norm(r)) {
        return Err(DiscreteError::NotPositiveDefinite);
    }
    Ok(())
}

/// The smallest real part among a matrix's eigenvalues (real for symmetric input).
fn min_eigenvalue(a: &Matrix) -> Result<f64, DiscreteError> {
    let decomposition = eigen(a)?;
    Ok(decomposition.values.iter().map(|value| value.re).fold(f64::INFINITY, f64::min))
}
