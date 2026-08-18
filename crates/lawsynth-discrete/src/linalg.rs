//! Small deterministic dense linear algebra for the discrete-time designs.
//!
//! Everything here is hand-rolled on top of [`Matrix`] with the standard library
//! only: elementwise combinators, a symmetrizer, norms, and Gauss–Jordan
//! inversion with partial pivoting (largest-magnitude pivot, lowest index on a
//! tie). Fixed loop order and fixed pivot rules make every result reproducible
//! to the bit.

use lawsynth_koopman::Matrix;

use crate::error::DiscreteError;

/// Multiplies two matrices, mapping a shape error into a [`DiscreteError`].
pub fn mm(a: &Matrix, b: &Matrix) -> Result<Matrix, DiscreteError> {
    a.matmul(b).map_err(|_| DiscreteError::ShapeMismatch)
}

/// The chained product `a · b · c`.
pub fn mm3(a: &Matrix, b: &Matrix, c: &Matrix) -> Result<Matrix, DiscreteError> {
    mm(&mm(a, b)?, c)
}

/// The elementwise sum `a + b` (shapes assumed equal).
pub fn add(a: &Matrix, b: &Matrix) -> Matrix {
    debug_assert_eq!((a.rows(), a.cols()), (b.rows(), b.cols()));
    let mut out = Matrix::zeros(a.rows(), a.cols());
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            out.set(i, j, a.get(i, j) + b.get(i, j));
        }
    }
    out
}

/// The elementwise difference `a − b` (shapes assumed equal).
pub fn sub(a: &Matrix, b: &Matrix) -> Matrix {
    debug_assert_eq!((a.rows(), a.cols()), (b.rows(), b.cols()));
    let mut out = Matrix::zeros(a.rows(), a.cols());
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            out.set(i, j, a.get(i, j) - b.get(i, j));
        }
    }
    out
}

/// The symmetric part `(a + aᵀ) / 2`, which removes tiny rounding asymmetry from
/// a matrix that is symmetric in exact arithmetic.
pub fn symmetrize(a: &Matrix) -> Matrix {
    let n = a.rows();
    let mut out = Matrix::zeros(n, n);
    for i in 0..n {
        for j in 0..n {
            out.set(i, j, 0.5 * (a.get(i, j) + a.get(j, i)));
        }
    }
    out
}

/// The Frobenius norm `sqrt(Σ aᵢⱼ²)`.
pub fn frobenius_norm(a: &Matrix) -> f64 {
    let mut sum = 0.0;
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            let value = a.get(i, j);
            sum += value * value;
        }
    }
    sum.sqrt()
}

/// The largest absolute entry of `a` (its max-norm).
pub fn max_abs(a: &Matrix) -> f64 {
    let mut best = 0.0_f64;
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            best = best.max(a.get(i, j).abs());
        }
    }
    best
}

/// The largest absolute entry of `a − b` (same shape assumed).
pub fn max_abs_diff(a: &Matrix, b: &Matrix) -> f64 {
    let mut best = 0.0_f64;
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            best = best.max((a.get(i, j) - b.get(i, j)).abs());
        }
    }
    best
}

/// True when `a` equals its transpose within `tol`.
pub fn is_symmetric(a: &Matrix, tol: f64) -> bool {
    let n = a.rows();
    if a.cols() != n {
        return false;
    }
    for i in 0..n {
        for j in i + 1..n {
            if (a.get(i, j) - a.get(j, i)).abs() > tol {
                return false;
            }
        }
    }
    true
}

/// True when every matrix entry is finite.
pub fn is_finite(a: &Matrix) -> bool {
    (0..a.rows()).all(|i| (0..a.cols()).all(|j| a.get(i, j).is_finite()))
}

/// Inverts a square matrix by Gauss–Jordan elimination with partial pivoting.
///
/// Returns [`DiscreteError::SingularSystem`] when the matrix is numerically
/// singular (a zero pivot column).
#[allow(clippy::needless_range_loop)]
pub fn invert(a: &Matrix) -> Result<Matrix, DiscreteError> {
    let n = a.rows();
    if a.cols() != n {
        return Err(DiscreteError::NonSquare);
    }
    // Augment `[A | I]` and reduce the left block to the identity.
    let mut work: Vec<Vec<f64>> = (0..n)
        .map(|i| {
            let mut row = vec![0.0; 2 * n];
            for j in 0..n {
                row[j] = a.get(i, j);
            }
            row[n + i] = 1.0;
            row
        })
        .collect();

    for col in 0..n {
        let mut pivot = col;
        let mut best = work[col][col].abs();
        for row in col + 1..n {
            let candidate = work[row][col].abs();
            if candidate > best {
                best = candidate;
                pivot = row;
            }
        }
        if best == 0.0 {
            return Err(DiscreteError::SingularSystem);
        }
        work.swap(col, pivot);

        let diagonal = work[col][col];
        for j in 0..2 * n {
            work[col][j] /= diagonal;
        }
        for row in 0..n {
            if row == col {
                continue;
            }
            let factor = work[row][col];
            if factor == 0.0 {
                continue;
            }
            for j in 0..2 * n {
                work[row][j] -= factor * work[col][j];
            }
        }
    }

    let mut inverse = Matrix::zeros(n, n);
    for i in 0..n {
        for j in 0..n {
            inverse.set(i, j, work[i][n + j]);
        }
    }
    Ok(inverse)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matrix(rows: Vec<Vec<f64>>) -> Matrix {
        Matrix::from_rows(rows).unwrap()
    }

    #[test]
    fn inverts_and_round_trips() {
        let a = matrix(vec![vec![4.0, 3.0], vec![6.0, 3.0]]);
        let inverse = invert(&a).unwrap();
        let product = mm(&a, &inverse).unwrap();
        assert!(max_abs_diff(&product, &Matrix::identity(2)) < 1e-12);
    }

    #[test]
    fn reports_singular_matrix() {
        let a = matrix(vec![vec![1.0, 2.0], vec![2.0, 4.0]]);
        assert_eq!(invert(&a).unwrap_err(), DiscreteError::SingularSystem);
    }

    #[test]
    fn symmetrize_removes_asymmetry() {
        let a = matrix(vec![vec![1.0, 2.0], vec![0.0, 3.0]]);
        let s = symmetrize(&a);
        assert_eq!(s.get(0, 1), 1.0);
        assert_eq!(s.get(1, 0), 1.0);
    }
}
