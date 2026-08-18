//! A small, deterministic real dense linear solver.
//!
//! Newton needs `J·δ = f(x)` solved at every step. We use Gaussian elimination
//! with partial pivoting on an owned copy of the matrix — standard-library only,
//! fixed pivot selection (largest magnitude in the column, lowest index on a
//! tie), so identical inputs give bit-identical solutions. A structurally or
//! numerically singular system returns `None` rather than a fabricated answer,
//! which lets the caller drop that Newton step honestly.

/// Solves `a · x = b` for `x` by Gaussian elimination with partial pivoting.
///
/// Returns `None` when the system is singular (a zero pivot column) or when the
/// solution is not finite. `a` must be a square `n × n` matrix and `b` length
/// `n`; a shape mismatch also yields `None`.
#[allow(clippy::needless_range_loop)]
pub(crate) fn solve_linear(a: &[Vec<f64>], b: &[f64]) -> Option<Vec<f64>> {
    let n = b.len();
    if a.len() != n || a.iter().any(|row| row.len() != n) {
        return None;
    }

    let mut work: Vec<Vec<f64>> = a.to_vec();
    let mut rhs = b.to_vec();

    for col in 0..n {
        // Partial pivot: the largest-magnitude entry at or below the diagonal.
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
            return None;
        }
        work.swap(col, pivot);
        rhs.swap(col, pivot);

        let diagonal = work[col][col];
        for row in col + 1..n {
            let factor = work[row][col] / diagonal;
            if factor == 0.0 {
                continue;
            }
            for c in col..n {
                work[row][c] -= factor * work[col][c];
            }
            rhs[row] -= factor * rhs[col];
        }
    }

    let mut solution = vec![0.0; n];
    for i in (0..n).rev() {
        let mut sum = rhs[i];
        for j in i + 1..n {
            sum -= work[i][j] * solution[j];
        }
        solution[i] = sum / work[i][i];
    }

    if solution.iter().all(|value| value.is_finite()) { Some(solution) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solves_a_two_by_two_system() {
        // [[2, 1], [1, 3]] x = [3, 5]  ->  x = [0.8, 1.4]
        let a = vec![vec![2.0, 1.0], vec![1.0, 3.0]];
        let b = vec![3.0, 5.0];
        let x = solve_linear(&a, &b).unwrap();
        assert!((x[0] - 0.8).abs() < 1e-12);
        assert!((x[1] - 1.4).abs() < 1e-12);
    }

    #[test]
    fn solves_a_system_needing_a_row_swap() {
        // A zero leading pivot forces partial pivoting.
        let a = vec![vec![0.0, 1.0], vec![1.0, 0.0]];
        let b = vec![2.0, 3.0];
        let x = solve_linear(&a, &b).unwrap();
        assert_eq!(x, vec![3.0, 2.0]);
    }

    #[test]
    fn reports_singular_system() {
        let a = vec![vec![1.0, 2.0], vec![2.0, 4.0]];
        let b = vec![1.0, 2.0];
        assert!(solve_linear(&a, &b).is_none());
    }

    #[test]
    fn rejects_shape_mismatch() {
        let a = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        assert!(solve_linear(&a, &[1.0]).is_none());
    }
}
