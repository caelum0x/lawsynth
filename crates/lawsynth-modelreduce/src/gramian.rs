//! Controllability and observability gramians of a stable linear system.
//!
//! For a Hurwitz `A`, the infinite-horizon gramians are the unique symmetric
//! positive-(semi)definite solutions of the continuous Lyapunov equations
//!
//! ```text
//! A Wc + Wc Aᵀ + B Bᵀ = 0        (controllability)
//! Aᵀ Wo + Wo A + Cᵀ C = 0        (observability)
//! ```
//!
//! Both are solved exactly by the deterministic vectorization solver in
//! [`crate::linalg`] and symmetrized to remove rounding asymmetry.

use lawsynth_koopman::Matrix;

use crate::error::ModelReduceError;
use crate::linalg::{lyapunov, mm, scale, symmetrize};

/// Solves `A Wc + Wc Aᵀ = −B Bᵀ` for the controllability gramian `Wc`.
///
/// Requires a Hurwitz `A` (checked upstream); the returned matrix is symmetric
/// and positive semidefinite in exact arithmetic.
pub fn controllability_gramian(a: &Matrix, b: &Matrix) -> Result<Matrix, ModelReduceError> {
    if a.rows() != a.cols() {
        return Err(ModelReduceError::NonSquareState);
    }
    if b.rows() != a.rows() {
        return Err(ModelReduceError::InputDimensionMismatch);
    }
    let bbt = mm(b, &b.transpose())?;
    let wc = lyapunov(a, &scale(&bbt, -1.0))?;
    Ok(symmetrize(&wc))
}

/// Solves `Aᵀ Wo + Wo A = −Cᵀ C` for the observability gramian `Wo`.
///
/// Requires a Hurwitz `A` (checked upstream); the returned matrix is symmetric
/// and positive semidefinite in exact arithmetic.
pub fn observability_gramian(a: &Matrix, c: &Matrix) -> Result<Matrix, ModelReduceError> {
    if a.rows() != a.cols() {
        return Err(ModelReduceError::NonSquareState);
    }
    if c.cols() != a.cols() {
        return Err(ModelReduceError::OutputDimensionMismatch);
    }
    let ctc = mm(&c.transpose(), c)?;
    // lyapunov(M, C) solves M X + X Mᵀ = C; with M = Aᵀ this is Aᵀ Wo + Wo A = C.
    let wo = lyapunov(&a.transpose(), &scale(&ctc, -1.0))?;
    Ok(symmetrize(&wo))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linalg::{add, max_abs_diff};

    #[test]
    fn controllability_gramian_satisfies_its_lyapunov_equation() {
        let a = Matrix::from_rows(vec![vec![-1.0, 0.0], vec![0.0, -2.0]]).unwrap();
        let b = Matrix::from_rows(vec![vec![1.0], vec![1.0]]).unwrap();
        let wc = controllability_gramian(&a, &b).unwrap();
        // Residual A Wc + Wc Aᵀ + B Bᵀ ≈ 0.
        let residual = add(
            &add(&mm(&a, &wc).unwrap(), &mm(&wc, &a.transpose()).unwrap()),
            &mm(&b, &b.transpose()).unwrap(),
        );
        assert!(max_abs_diff(&residual, &Matrix::zeros(2, 2)) < 1e-12);
    }

    #[test]
    fn scalar_gramian_is_one_half() {
        // ẋ = −x, B = 1 ⇒ Wc solves −2 Wc + 1 = 0 ⇒ Wc = 1/2.
        let a = Matrix::from_rows(vec![vec![-1.0]]).unwrap();
        let b = Matrix::from_rows(vec![vec![1.0]]).unwrap();
        let wc = controllability_gramian(&a, &b).unwrap();
        assert!((wc.get(0, 0) - 0.5).abs() < 1e-15);
    }
}
