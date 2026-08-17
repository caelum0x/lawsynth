//! A deterministic economy singular value decomposition.
//!
//! The algorithm is the classic *one-sided Jacobi* method (Golub & Van Loan,
//! §8.6.3): it orthogonalises the columns of the working matrix by a fixed,
//! cyclic sequence of plane rotations. It is chosen because it is compact,
//! self-contained (no external crate), backward-stable, and — with a fixed
//! sweep order and convergence tolerance — bit-for-bit reproducible.
//!
//! One-sided Jacobi needs a matrix with at least as many rows as columns, so an
//! input with fewer rows than columns is transposed first and the factors are
//! swapped on the way out.

use crate::{KoopmanError, Matrix};

/// An economy SVD `A = U · diag(s) · Vᵀ`.
///
/// For an input `p × q` matrix with `k = min(p, q)`: `u` is `p × k` with
/// orthonormal columns, `s` holds the `k` singular values in non-increasing
/// order, and `v` is `q × k` with orthonormal columns.
#[derive(Clone, Debug)]
pub struct Svd {
    pub u: Matrix,
    pub s: Vec<f64>,
    pub v: Matrix,
}

const MAX_SWEEPS: usize = 80;
const ORTHO_TOLERANCE: f64 = 1e-15;

/// Computes the economy SVD of `matrix` deterministically.
pub fn svd(matrix: &Matrix) -> Result<Svd, KoopmanError> {
    if matrix.rows() >= matrix.cols() {
        jacobi_tall(matrix)
    } else {
        // Fewer rows than columns: decompose the transpose, then swap U and V.
        let transposed = matrix.transpose();
        let Svd { u, s, v } = jacobi_tall(&transposed)?;
        Ok(Svd { u: v, s, v: u })
    }
}

/// One-sided Jacobi on a matrix with `rows >= cols`.
fn jacobi_tall(matrix: &Matrix) -> Result<Svd, KoopmanError> {
    let rows = matrix.rows();
    let cols = matrix.cols();

    // Working copy whose columns are rotated to mutual orthogonality.
    let mut work: Vec<Vec<f64>> =
        (0..cols).map(|col| (0..rows).map(|row| matrix.get(row, col)).collect()).collect();
    // Accumulated right factor (starts as the identity).
    let mut right: Vec<Vec<f64>> =
        (0..cols).map(|i| (0..cols).map(|j| if i == j { 1.0 } else { 0.0 }).collect()).collect();

    let mut converged = false;
    for _ in 0..MAX_SWEEPS {
        let mut rotated = false;
        for i in 0..cols {
            for j in (i + 1)..cols {
                let (alpha, beta, gamma) = column_products(&work, i, j);
                if gamma.abs() <= ORTHO_TOLERANCE * (alpha * beta).sqrt() {
                    continue;
                }
                let (cos, sin) = jacobi_rotation(alpha, beta, gamma);
                rotate_columns(&mut work, i, j, cos, sin);
                rotate_columns(&mut right, i, j, cos, sin);
                rotated = true;
            }
        }
        if !rotated {
            converged = true;
            break;
        }
    }
    if !converged {
        return Err(KoopmanError::NoConvergence);
    }

    finalize(work, right, rows, cols)
}

/// Inner products `(cᵢ·cᵢ, cⱼ·cⱼ, cᵢ·cⱼ)` of two working columns.
fn column_products(work: &[Vec<f64>], i: usize, j: usize) -> (f64, f64, f64) {
    let (col_i, col_j) = (&work[i], &work[j]);
    let mut alpha = 0.0;
    let mut beta = 0.0;
    let mut gamma = 0.0;
    for (a, b) in col_i.iter().zip(col_j) {
        alpha += a * a;
        beta += b * b;
        gamma += a * b;
    }
    (alpha, beta, gamma)
}

/// The `(cos, sin)` of the plane rotation that diagonalises `[[α, γ], [γ, β]]`.
fn jacobi_rotation(alpha: f64, beta: f64, gamma: f64) -> (f64, f64) {
    let zeta = (beta - alpha) / (2.0 * gamma);
    let tan = zeta.signum() / (zeta.abs() + (1.0 + zeta * zeta).sqrt());
    let cos = 1.0 / (1.0 + tan * tan).sqrt();
    (cos, cos * tan)
}

/// Applies a plane rotation to columns `i` and `j` (with `i < j`) in place.
fn rotate_columns(columns: &mut [Vec<f64>], i: usize, j: usize, cos: f64, sin: f64) {
    let (left_part, right_part) = columns.split_at_mut(j);
    let col_i = &mut left_part[i];
    let col_j = &mut right_part[0];
    for (a, b) in col_i.iter_mut().zip(col_j.iter_mut()) {
        let left = *a;
        let right = *b;
        *a = cos * left - sin * right;
        *b = sin * left + cos * right;
    }
}

/// Extracts singular values, sorts them, and assembles the ordered factors.
fn finalize(
    work: Vec<Vec<f64>>,
    right: Vec<Vec<f64>>,
    rows: usize,
    cols: usize,
) -> Result<Svd, KoopmanError> {
    // Singular values are the norms of the orthogonalised columns.
    let mut singular: Vec<(usize, f64)> = work
        .iter()
        .enumerate()
        .map(|(index, col)| (index, col.iter().map(|v| v * v).sum::<f64>().sqrt()))
        .collect();
    // Deterministic descending order, breaking ties by original column index.
    singular.sort_by(|left, right| right.1.total_cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

    let mut u = Matrix::zeros(rows, cols);
    let mut v = Matrix::zeros(cols, cols);
    let mut s = vec![0.0; cols];
    for (output_col, &(source_col, sigma)) in singular.iter().enumerate() {
        s[output_col] = sigma;
        // U column is the normalised working column; a numerically-zero column
        // contributes no direction and is left as zeros.
        if sigma > 0.0 {
            for (row, &value) in work[source_col].iter().enumerate() {
                u.set(row, output_col, value / sigma);
            }
        }
        for (row, &value) in right[source_col].iter().enumerate() {
            v.set(row, output_col, value);
        }
    }
    Ok(Svd { u, s, v })
}
