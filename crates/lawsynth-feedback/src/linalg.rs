//! Small deterministic dense linear algebra used by the feedback designs.
//!
//! Everything here is hand-rolled on top of [`Matrix`] with the standard library
//! only: Gaussian elimination with partial pivoting (largest-magnitude pivot,
//! lowest index on a tie), matrix inversion, a Kronecker product, and a
//! continuous Lyapunov solver built by vectorization. Fixed loop order and fixed
//! pivot rules make every result reproducible to the bit.

use lawsynth_koopman::Matrix;

use crate::error::FeedbackError;

/// Multiplies two matrices, mapping a shape error into a [`FeedbackError`].
pub fn mm(a: &Matrix, b: &Matrix) -> Result<Matrix, FeedbackError> {
    a.matmul(b).map_err(|_| FeedbackError::ShapeMismatch)
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

/// The scalar multiple `factor · a`.
pub fn scale(a: &Matrix, factor: f64) -> Matrix {
    let mut out = Matrix::zeros(a.rows(), a.cols());
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            out.set(i, j, a.get(i, j) * factor);
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

/// The Kronecker product `a ⊗ b`.
pub fn kron(a: &Matrix, b: &Matrix) -> Matrix {
    let (ar, ac) = (a.rows(), a.cols());
    let (br, bc) = (b.rows(), b.cols());
    let mut out = Matrix::zeros(ar * br, ac * bc);
    for i in 0..ar {
        for j in 0..ac {
            let scalar = a.get(i, j);
            if scalar == 0.0 {
                continue;
            }
            for k in 0..br {
                for l in 0..bc {
                    out.set(i * br + k, j * bc + l, scalar * b.get(k, l));
                }
            }
        }
    }
    out
}

/// Inverts a square matrix by Gauss–Jordan elimination with partial pivoting.
///
/// Returns [`FeedbackError::SingularSystem`] when the matrix is numerically
/// singular (a zero pivot column).
#[allow(clippy::needless_range_loop)]
pub fn invert(a: &Matrix) -> Result<Matrix, FeedbackError> {
    let n = a.rows();
    if a.cols() != n {
        return Err(FeedbackError::NonSquare);
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
            return Err(FeedbackError::SingularSystem);
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

/// Solves the dense square system `a · x = rhs` by Gaussian elimination with
/// partial pivoting and back-substitution.
#[allow(clippy::needless_range_loop)]
pub fn solve_dense(a: &Matrix, rhs: &[f64]) -> Result<Vec<f64>, FeedbackError> {
    let n = a.rows();
    if a.cols() != n || rhs.len() != n {
        return Err(FeedbackError::ShapeMismatch);
    }
    let mut work: Vec<Vec<f64>> = (0..n).map(|i| (0..n).map(|j| a.get(i, j)).collect()).collect();
    let mut vector = rhs.to_vec();

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
            return Err(FeedbackError::SingularSystem);
        }
        work.swap(col, pivot);
        vector.swap(col, pivot);

        let diagonal = work[col][col];
        for row in col + 1..n {
            let factor = work[row][col] / diagonal;
            if factor == 0.0 {
                continue;
            }
            for j in col..n {
                work[row][j] -= factor * work[col][j];
            }
            vector[row] -= factor * vector[col];
        }
    }

    let mut solution = vec![0.0; n];
    for i in (0..n).rev() {
        let mut sum = vector[i];
        for j in i + 1..n {
            sum -= work[i][j] * solution[j];
        }
        solution[i] = sum / work[i][i];
    }
    Ok(solution)
}

/// Solves the continuous Lyapunov equation `M X + X Mᵀ = C` for `X`.
///
/// The equation is vectorized (column-major) into the `n² × n²` linear system
/// `(I ⊗ M + M ⊗ I) vec(X) = vec(C)` and solved with [`solve_dense`]. This is
/// exact and deterministic; it has a unique solution iff `M` shares no
/// eigenvalue with `−Mᵀ` (i.e. `λᵢ + λⱼ ≠ 0`), otherwise the dense solve
/// reports [`FeedbackError::SingularSystem`].
pub fn lyapunov(m: &Matrix, c: &Matrix) -> Result<Matrix, FeedbackError> {
    let n = m.rows();
    if m.cols() != n || c.rows() != n || c.cols() != n {
        return Err(FeedbackError::ShapeMismatch);
    }
    let identity = Matrix::identity(n);
    let system = add(&kron(&identity, m), &kron(m, &identity));

    // Column-major vec(C): entry (i, j) lands at index j·n + i.
    let mut rhs = vec![0.0; n * n];
    for j in 0..n {
        for i in 0..n {
            rhs[j * n + i] = c.get(i, j);
        }
    }

    let solved = solve_dense(&system, &rhs)?;

    let mut x = Matrix::zeros(n, n);
    for j in 0..n {
        for i in 0..n {
            x.set(i, j, solved[j * n + i]);
        }
    }
    Ok(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inverts_and_round_trips() {
        let a = Matrix::from_rows(vec![vec![4.0, 3.0], vec![6.0, 3.0]]).unwrap();
        let inverse = invert(&a).unwrap();
        let product = mm(&a, &inverse).unwrap();
        let identity = Matrix::identity(2);
        assert!(max_abs_diff(&product, &identity) < 1e-12);
    }

    #[test]
    fn reports_singular_matrix() {
        let a = Matrix::from_rows(vec![vec![1.0, 2.0], vec![2.0, 4.0]]).unwrap();
        assert_eq!(invert(&a).unwrap_err(), FeedbackError::SingularSystem);
    }

    #[test]
    fn solves_lyapunov_against_a_known_solution() {
        // Choose a stable M and X ≻ 0, form C = M X + X Mᵀ, recover X.
        let m = Matrix::from_rows(vec![vec![-2.0, 1.0], vec![0.0, -3.0]]).unwrap();
        let x = Matrix::from_rows(vec![vec![2.0, 0.5], vec![0.5, 1.0]]).unwrap();
        let c = add(&mm(&m, &x).unwrap(), &mm(&x, &m.transpose()).unwrap());
        let recovered = lyapunov(&m, &c).unwrap();
        assert!(max_abs_diff(&recovered, &x) < 1e-10);
    }

    #[test]
    fn kron_has_expected_shape_and_entries() {
        let a = Matrix::from_rows(vec![vec![1.0, 2.0], vec![3.0, 4.0]]).unwrap();
        let b = Matrix::identity(2);
        let product = kron(&a, &b);
        assert_eq!((product.rows(), product.cols()), (4, 4));
        assert_eq!(product.get(0, 0), 1.0);
        assert_eq!(product.get(1, 1), 1.0);
        assert_eq!(product.get(2, 0), 3.0);
    }
}
