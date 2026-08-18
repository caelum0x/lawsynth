//! Square-root balancing (Laub's method) of a stable linear realization.
//!
//! Given the gramians `Wc` and `Wo`, the balancing transform `T` puts the system
//! into coordinates where both gramians equal the same diagonal matrix `Σ` of
//! **Hankel singular values**. The construction is the numerically sound
//! square-root form:
//!
//! 1. Factor `Wc = R Rᵀ` from its symmetric eigendecomposition
//!    `Wc = Q diag(λ) Qᵀ`, so `R = Q diag(√λ)`.
//! 2. Diagonalize `M = Rᵀ Wo R = U diag(σ²) Uᵀ`; the Hankel singular values are
//!    `σ = √diag(σ²)`.
//! 3. Set `T = R U Σ^{-1/2}` and `T⁻¹ = Σ^{1/2} Uᵀ R⁻¹`, where `R⁻¹ = diag(1/√λ) Qᵀ`
//!    is available in closed form from the orthogonal `Q`.
//!
//! Then `T⁻¹ Wc T⁻ᵀ = Tᵀ Wo T = Σ`, which the balanced-realization test verifies.

use lawsynth_koopman::Matrix;

use crate::error::ModelReduceError;
use crate::gramian::{controllability_gramian, observability_gramian};
use crate::linalg::{diag_times_mat, mat_times_diag, mm, symmetric_eigen, symmetrize};

/// A balancing transform and the Hankel singular values it exposes.
pub(crate) struct Balancing {
    /// Hankel singular values in non-increasing order, length `n`.
    pub(crate) sigma: Vec<f64>,
    /// The balancing transform `T` (`x_original = T · x_balanced`).
    pub(crate) t: Matrix,
    /// Its inverse `T⁻¹` (`x_balanced = T⁻¹ · x_original`).
    pub(crate) t_inv: Matrix,
}

/// Computes the square-root balancing transform for a stable realization.
pub(crate) fn balancing_transform(
    a: &Matrix,
    b: &Matrix,
    c: &Matrix,
) -> Result<Balancing, ModelReduceError> {
    let n = a.rows();
    let wc = controllability_gramian(a, b)?;
    let wo = observability_gramian(a, c)?;

    // Step 1: Wc = Q diag(λ) Qᵀ ⇒ R = Q diag(√λ), R⁻¹ = diag(1/√λ) Qᵀ.
    let wc_eig = symmetric_eigen(&wc)?;
    let mut root = vec![0.0; n];
    let mut inv_root = vec![0.0; n];
    for i in 0..n {
        if wc_eig.values[i] <= 0.0 {
            // A non-positive controllability eigenvalue means an uncontrollable
            // direction: the balancing transform is singular.
            return Err(ModelReduceError::SingularSystem);
        }
        root[i] = wc_eig.values[i].sqrt();
        inv_root[i] = 1.0 / root[i];
    }
    let r = mat_times_diag(&wc_eig.q, &root);
    let r_inv = diag_times_mat(&inv_root, &wc_eig.q.transpose());

    // Step 2: M = Rᵀ Wo R = U diag(σ²) Uᵀ.
    let m = symmetrize(&mm(&mm(&r.transpose(), &wo)?, &r)?);
    let m_eig = symmetric_eigen(&m)?;

    let mut sigma = vec![0.0; n];
    let mut sigma_inv_sqrt = vec![0.0; n];
    let mut sigma_sqrt = vec![0.0; n];
    for i in 0..n {
        let value = m_eig.values[i].max(0.0);
        sigma[i] = value.sqrt();
        if sigma[i] <= 0.0 {
            // A zero Hankel singular value: a mode both uncontrollable and
            // unobservable; Σ^{-1/2} is undefined, so balancing fails.
            return Err(ModelReduceError::SingularSystem);
        }
        sigma_sqrt[i] = sigma[i].sqrt();
        sigma_inv_sqrt[i] = 1.0 / sigma_sqrt[i];
    }

    // Step 3: T = R U Σ^{-1/2},  T⁻¹ = Σ^{1/2} Uᵀ R⁻¹.
    let t = mat_times_diag(&mm(&r, &m_eig.q)?, &sigma_inv_sqrt);
    let t_inv = diag_times_mat(&sigma_sqrt, &mm(&m_eig.q.transpose(), &r_inv)?);

    Ok(Balancing { sigma, t, t_inv })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linalg::max_abs_diff;

    #[test]
    fn transform_and_inverse_are_consistent() {
        let a = Matrix::from_rows(vec![vec![-1.0, 0.0], vec![0.0, -2.0]]).unwrap();
        let b = Matrix::from_rows(vec![vec![1.0], vec![1.0]]).unwrap();
        let c = Matrix::from_rows(vec![vec![1.0, 1.0]]).unwrap();
        let balancing = balancing_transform(&a, &b, &c).unwrap();
        let product = mm(&balancing.t_inv, &balancing.t).unwrap();
        assert!(max_abs_diff(&product, &Matrix::identity(2)) < 1e-10);
    }
}
