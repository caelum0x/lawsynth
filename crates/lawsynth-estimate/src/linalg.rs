//! Small deterministic dense helpers shared by observer design and simulation.
//!
//! Everything is hand-rolled on top of the shared [`Matrix`] and plain `f64`
//! slices with the standard library only. Fixed loop order and a fixed pivot
//! rule (largest magnitude, lowest index on a tie) keep every result
//! reproducible to the bit.

use lawsynth_koopman::Matrix;

use crate::error::EstimateError;

/// Multiplies two matrices, mapping a shape error into an [`EstimateError`].
pub(crate) fn mm(a: &Matrix, b: &Matrix) -> Result<Matrix, EstimateError> {
    a.matmul(b).map_err(|_| EstimateError::ShapeMismatch)
}

/// The elementwise difference `a − b` (shapes assumed equal).
pub(crate) fn sub(a: &Matrix, b: &Matrix) -> Matrix {
    let mut out = Matrix::zeros(a.rows(), a.cols());
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            out.set(i, j, a.get(i, j) - b.get(i, j));
        }
    }
    out
}

/// True when every matrix entry is finite.
pub(crate) fn is_finite(a: &Matrix) -> bool {
    (0..a.rows()).all(|i| (0..a.cols()).all(|j| a.get(i, j).is_finite()))
}

/// The largest absolute entry of `a` (its max-norm).
pub(crate) fn max_abs(a: &Matrix) -> f64 {
    let mut best = 0.0_f64;
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            best = best.max(a.get(i, j).abs());
        }
    }
    best
}

/// The error dynamics `A − L C`, whose eigenvalues are the observer error poles.
pub(crate) fn error_dynamics(a: &Matrix, l: &Matrix, c: &Matrix) -> Result<Matrix, EstimateError> {
    Ok(sub(a, &mm(l, c)?))
}

/// The column rank of `m` by Gaussian elimination with partial pivoting.
///
/// A column contributes to the rank when, after eliminating the already-found
/// pivot rows, it still carries a pivot above `tol`. Used to test full column
/// rank of the observability matrix.
#[allow(clippy::needless_range_loop)]
pub(crate) fn column_rank(m: &Matrix, tol: f64) -> usize {
    let rows = m.rows();
    let cols = m.cols();
    let mut work: Vec<Vec<f64>> =
        (0..rows).map(|i| (0..cols).map(|j| m.get(i, j)).collect()).collect();

    let mut rank = 0;
    let mut pivot_row = 0;
    for col in 0..cols {
        if pivot_row >= rows {
            break;
        }
        let mut best = pivot_row;
        let mut best_val = work[pivot_row][col].abs();
        for row in pivot_row + 1..rows {
            let candidate = work[row][col].abs();
            if candidate > best_val {
                best_val = candidate;
                best = row;
            }
        }
        if best_val <= tol {
            continue;
        }
        work.swap(pivot_row, best);

        let pivot = work[pivot_row][col];
        for row in pivot_row + 1..rows {
            let factor = work[row][col] / pivot;
            if factor == 0.0 {
                continue;
            }
            for c in col..cols {
                work[row][c] -= factor * work[pivot_row][c];
            }
        }
        rank += 1;
        pivot_row += 1;
    }
    rank
}

/// The product `a · x` with a column vector, mapping shape errors.
pub(crate) fn mat_vec(a: &Matrix, x: &[f64]) -> Result<Vec<f64>, EstimateError> {
    a.mat_vec(x).map_err(|_| EstimateError::ShapeMismatch)
}

/// The Euclidean norm `‖x‖₂`.
pub(crate) fn norm2(x: &[f64]) -> f64 {
    x.iter().map(|value| value * value).sum::<f64>().sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_rank_and_deficient() {
        let full = Matrix::from_rows(vec![vec![1.0, 0.0], vec![0.0, 1.0], vec![1.0, 1.0]]).unwrap();
        assert_eq!(column_rank(&full, 1e-9), 2);
        let deficient = Matrix::from_rows(vec![vec![1.0, 2.0], vec![2.0, 4.0]]).unwrap();
        assert_eq!(column_rank(&deficient, 1e-9), 1);
    }

    #[test]
    fn error_dynamics_subtracts_outer_product() {
        let a = Matrix::from_rows(vec![vec![0.0, 1.0], vec![0.0, 0.0]]).unwrap();
        let l = Matrix::from_rows(vec![vec![5.0], vec![6.0]]).unwrap();
        let c = Matrix::from_rows(vec![vec![1.0, 0.0]]).unwrap();
        let e = error_dynamics(&a, &l, &c).unwrap();
        assert_eq!(e.get(0, 0), -5.0);
        assert_eq!(e.get(0, 1), 1.0);
        assert_eq!(e.get(1, 0), -6.0);
        assert_eq!(e.get(1, 1), 0.0);
    }
}
