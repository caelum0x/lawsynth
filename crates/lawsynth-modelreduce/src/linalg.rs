//! Small deterministic dense linear algebra for balanced truncation.
//!
//! Everything here is hand-rolled on top of [`Matrix`] with the standard library
//! only: matrix products and sums, a Kronecker product, Gaussian elimination with
//! partial pivoting (largest-magnitude pivot, lowest index on a tie), a continuous
//! Lyapunov solver built by vectorization, and a cyclic Jacobi eigensolver for
//! real symmetric matrices. Every routine has a fixed loop and pivot order, so
//! identical inputs yield bit-identical output.

use lawsynth_koopman::Matrix;

use crate::error::ModelReduceError;

/// Multiplies two matrices, mapping a shape error into a [`ModelReduceError`].
pub(crate) fn mm(a: &Matrix, b: &Matrix) -> Result<Matrix, ModelReduceError> {
    a.matmul(b).map_err(|_| ModelReduceError::ShapeMismatch)
}

/// The elementwise sum `a + b` (equal shapes assumed).
pub(crate) fn add(a: &Matrix, b: &Matrix) -> Matrix {
    debug_assert_eq!((a.rows(), a.cols()), (b.rows(), b.cols()));
    let mut out = Matrix::zeros(a.rows(), a.cols());
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            out.set(i, j, a.get(i, j) + b.get(i, j));
        }
    }
    out
}

/// The scalar multiple `factor · a`.
pub(crate) fn scale(a: &Matrix, factor: f64) -> Matrix {
    let mut out = Matrix::zeros(a.rows(), a.cols());
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            out.set(i, j, a.get(i, j) * factor);
        }
    }
    out
}

/// The symmetric part `(a + aᵀ) / 2`, removing tiny rounding asymmetry from a
/// matrix that is symmetric in exact arithmetic.
pub(crate) fn symmetrize(a: &Matrix) -> Matrix {
    let n = a.rows();
    let mut out = Matrix::zeros(n, n);
    for i in 0..n {
        for j in 0..n {
            out.set(i, j, 0.5 * (a.get(i, j) + a.get(j, i)));
        }
    }
    out
}

/// Scales column `j` of `a` by `d[j]` (right multiply by `diag(d)`).
#[allow(clippy::needless_range_loop)]
pub(crate) fn mat_times_diag(a: &Matrix, d: &[f64]) -> Matrix {
    let mut out = Matrix::zeros(a.rows(), a.cols());
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            out.set(i, j, a.get(i, j) * d[j]);
        }
    }
    out
}

/// Scales row `i` of `a` by `d[i]` (left multiply by `diag(d)`).
#[allow(clippy::needless_range_loop)]
pub(crate) fn diag_times_mat(d: &[f64], a: &Matrix) -> Matrix {
    let mut out = Matrix::zeros(a.rows(), a.cols());
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            out.set(i, j, d[i] * a.get(i, j));
        }
    }
    out
}

/// The largest absolute entry of `a − b` (same shape assumed).
#[cfg(test)]
pub(crate) fn max_abs_diff(a: &Matrix, b: &Matrix) -> f64 {
    let mut best = 0.0_f64;
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            best = best.max((a.get(i, j) - b.get(i, j)).abs());
        }
    }
    best
}

/// The Kronecker product `a ⊗ b`.
pub(crate) fn kron(a: &Matrix, b: &Matrix) -> Matrix {
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

/// Solves the dense square system `a · x = rhs` by Gaussian elimination with
/// partial pivoting and back-substitution.
#[allow(clippy::needless_range_loop)]
pub(crate) fn solve_dense(a: &Matrix, rhs: &[f64]) -> Result<Vec<f64>, ModelReduceError> {
    let n = a.rows();
    if a.cols() != n || rhs.len() != n {
        return Err(ModelReduceError::ShapeMismatch);
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
            return Err(ModelReduceError::SingularSystem);
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
/// exact and deterministic; the system is non-singular iff `M` shares no
/// eigenvalue with `−Mᵀ` (i.e. `λᵢ + λⱼ ≠ 0`), otherwise the dense solve reports
/// [`ModelReduceError::SingularSystem`].
pub(crate) fn lyapunov(m: &Matrix, c: &Matrix) -> Result<Matrix, ModelReduceError> {
    let n = m.rows();
    if m.cols() != n || c.rows() != n || c.cols() != n {
        return Err(ModelReduceError::ShapeMismatch);
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

const JACOBI_MAX_SWEEPS: usize = 100;
const JACOBI_TOLERANCE: f64 = 1e-15;

/// A symmetric eigendecomposition `A = Q · diag(values) · Qᵀ`.
///
/// `values` are the eigenvalues in **non-increasing** order and `q` holds the
/// corresponding orthonormal eigenvectors as its columns. Signs are canonicalized
/// (the largest-magnitude component of each eigenvector is made positive) so the
/// factorization is bit-reproducible.
pub(crate) struct SymmetricEigen {
    pub(crate) values: Vec<f64>,
    pub(crate) q: Matrix,
}

/// Diagonalizes a real symmetric matrix by the cyclic Jacobi method.
///
/// The classic sweep rotates each off-diagonal `(p, q)` pair (in fixed order) to
/// zero, accumulating the rotations into `Q`. It is compact, backward-stable, and
/// — with a fixed sweep order and tolerance — bit-for-bit reproducible.
#[allow(clippy::needless_range_loop)]
pub(crate) fn symmetric_eigen(input: &Matrix) -> Result<SymmetricEigen, ModelReduceError> {
    let n = input.rows();
    if input.cols() != n {
        return Err(ModelReduceError::ShapeMismatch);
    }
    if n == 0 {
        return Err(ModelReduceError::EmptyMatrix);
    }
    if n == 1 {
        return Ok(SymmetricEigen { values: vec![input.get(0, 0)], q: Matrix::identity(1) });
    }

    let mut a: Vec<Vec<f64>> = (0..n).map(|i| (0..n).map(|j| input.get(i, j)).collect()).collect();
    let mut v: Vec<Vec<f64>> =
        (0..n).map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect()).collect();

    // Off-diagonal magnitudes are compared against a fixed threshold derived from
    // the (rotation-invariant) Frobenius norm, so convergence is scale-aware.
    let frobenius = {
        let mut sum = 0.0;
        for i in 0..n {
            for j in 0..n {
                sum += a[i][j] * a[i][j];
            }
        }
        sum.sqrt()
    };
    let threshold = JACOBI_TOLERANCE * frobenius;

    let mut converged = false;
    for _ in 0..JACOBI_MAX_SWEEPS {
        let mut rotated = false;
        for p in 0..n {
            for q in p + 1..n {
                if a[p][q].abs() <= threshold {
                    continue;
                }
                let (cos, sin) = jacobi_rotation(a[p][p], a[q][q], a[p][q]);
                apply_symmetric_rotation(&mut a, p, q, cos, sin);
                rotate_columns(&mut v, p, q, cos, sin);
                rotated = true;
            }
        }
        if !rotated {
            converged = true;
            break;
        }
    }
    if !converged {
        return Err(ModelReduceError::NoConvergence);
    }

    finalize_symmetric(a, v, n)
}

/// The `(cos, sin)` of the Jacobi rotation that annihilates the `(p, q)` entry of
/// a symmetric matrix with diagonal `(app, aqq)` and off-diagonal `apq ≠ 0`.
fn jacobi_rotation(app: f64, aqq: f64, apq: f64) -> (f64, f64) {
    let theta = (aqq - app) / (2.0 * apq);
    // `theta == 0` (equal diagonals) gives the 45° rotation `t = 1`.
    let tan = if theta >= 0.0 {
        1.0 / (theta + (theta * theta + 1.0).sqrt())
    } else {
        -1.0 / (-theta + (theta * theta + 1.0).sqrt())
    };
    let cos = 1.0 / (tan * tan + 1.0).sqrt();
    (cos, tan * cos)
}

/// Applies the two-sided update `A ← Jᵀ A J` for a rotation in the `(p, q)` plane.
#[allow(clippy::needless_range_loop)]
fn apply_symmetric_rotation(a: &mut [Vec<f64>], p: usize, q: usize, cos: f64, sin: f64) {
    let n = a.len();
    // Right multiply `A J`: recombine columns p and q.
    for i in 0..n {
        let aip = a[i][p];
        let aiq = a[i][q];
        a[i][p] = cos * aip - sin * aiq;
        a[i][q] = sin * aip + cos * aiq;
    }
    // Left multiply `Jᵀ (A J)`: recombine rows p and q.
    for j in 0..n {
        let apj = a[p][j];
        let aqj = a[q][j];
        a[p][j] = cos * apj - sin * aqj;
        a[q][j] = sin * apj + cos * aqj;
    }
}

/// Accumulates a plane rotation into the eigenvector matrix (`V ← V J`).
fn rotate_columns(v: &mut [Vec<f64>], p: usize, q: usize, cos: f64, sin: f64) {
    for row in v.iter_mut() {
        let vp = row[p];
        let vq = row[q];
        row[p] = cos * vp - sin * vq;
        row[q] = sin * vp + cos * vq;
    }
}

/// Extracts eigenvalues from the diagonal, sorts descending, and canonicalizes
/// eigenvector signs.
#[allow(clippy::needless_range_loop)]
fn finalize_symmetric(
    a: Vec<Vec<f64>>,
    v: Vec<Vec<f64>>,
    n: usize,
) -> Result<SymmetricEigen, ModelReduceError> {
    // Deterministic descending order, ties broken by original index.
    let mut order: Vec<(usize, f64)> = (0..n).map(|i| (i, a[i][i])).collect();
    order.sort_by(|left, right| right.1.total_cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

    let mut values = vec![0.0; n];
    let mut q = Matrix::zeros(n, n);
    for (out_col, &(src, value)) in order.iter().enumerate() {
        values[out_col] = value;
        // Sign canonicalization: make the largest-magnitude component positive.
        let mut pivot_row = 0;
        let mut best = 0.0_f64;
        for row in 0..n {
            let magnitude = v[row][src].abs();
            if magnitude > best {
                best = magnitude;
                pivot_row = row;
            }
        }
        let sign = if v[pivot_row][src] < 0.0 { -1.0 } else { 1.0 };
        for row in 0..n {
            q.set(row, out_col, sign * v[row][src]);
        }
    }
    Ok(SymmetricEigen { values, q })
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(product.get(2, 0), 3.0);
    }

    #[test]
    fn symmetric_eigen_reconstructs_the_matrix() {
        let a =
            Matrix::from_rows(vec![vec![2.0, 1.0, 0.0], vec![1.0, 2.0, 1.0], vec![0.0, 1.0, 2.0]])
                .unwrap();
        let decomposition = symmetric_eigen(&a).unwrap();
        // Q diag(values) Qᵀ == A.
        let diag = mat_times_diag(&decomposition.q, &decomposition.values);
        let reconstructed = mm(&diag, &decomposition.q.transpose()).unwrap();
        assert!(max_abs_diff(&reconstructed, &a) < 1e-12);
        // Eigenvalues of this matrix are 2, 2 ± √2; check descending order.
        assert!(decomposition.values[0] >= decomposition.values[1]);
        assert!(decomposition.values[1] >= decomposition.values[2]);
        assert!((decomposition.values[0] - (2.0 + 2.0_f64.sqrt())).abs() < 1e-10);
        assert!((decomposition.values[2] - (2.0 - 2.0_f64.sqrt())).abs() < 1e-10);
    }

    #[test]
    fn symmetric_eigen_has_orthonormal_vectors() {
        let a = Matrix::from_rows(vec![vec![4.0, 1.0], vec![1.0, 3.0]]).unwrap();
        let decomposition = symmetric_eigen(&a).unwrap();
        let gram = mm(&decomposition.q.transpose(), &decomposition.q).unwrap();
        assert!(max_abs_diff(&gram, &Matrix::identity(2)) < 1e-12);
    }
}
