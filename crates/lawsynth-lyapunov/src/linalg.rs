//! Small, std-only dense linear algebra for the variational integrator.
//!
//! The only decomposition the Benettin method needs is a QR factorization of the
//! evolved perturbation frame: `Q = Q'·R`, keeping the orthonormal `Q'` as the
//! new frame and the diagonal of `R` for the log-accumulation of the exponents.
//! A **modified** Gram–Schmidt is used — it is numerically better conditioned
//! than the classical variant while remaining fully deterministic (a fixed
//! column-then-row traversal, no pivoting, no RNG).

use crate::error::LyapunovError;

/// Columns whose Euclidean norm falls at or below this bound are treated as a
/// collapsed direction: the frame has degenerated and cannot be normalized.
const MIN_COLUMN_NORM: f64 = 1e-300;

/// Reorthonormalizes a perturbation frame in place with modified Gram–Schmidt.
///
/// The frame is passed as its columns `columns[j]`, each an `n`-vector. On
/// success the columns are overwritten with an orthonormal set spanning the same
/// nested subspaces (`span(q_0..q_i)` is preserved for every `i`), and the
/// returned vector holds the diagonal of `R` — `r[i]` is the length of column
/// `i` after the earlier columns have been projected out. Those `r[i]` are the
/// per-step local expansion factors whose logarithms Benettin's method averages.
///
/// The column order is the accumulation order of the exponents, so it must stay
/// fixed; nothing here depends on hash-map iteration or wall-clock state.
///
/// # Errors
///
/// Returns [`LyapunovError::DegenerateFrame`] if any column collapses to a
/// (numerically) zero length, and [`LyapunovError::NonFiniteState`] if a
/// non-finite value is encountered.
pub(crate) fn gram_schmidt_qr(columns: &mut [Vec<f64>]) -> Result<Vec<f64>, LyapunovError> {
    let n = columns.len();
    let mut r_diagonal = Vec::with_capacity(n);

    for i in 0..n {
        // r_ii is the norm of column i *after* the previous columns have already
        // been subtracted from it in earlier iterations (modified Gram–Schmidt).
        // `euclidean_norm` has already rejected any non-finite entry, so `norm` is
        // finite and this comparison is well-defined.
        let norm = euclidean_norm(&columns[i])?;
        if norm <= MIN_COLUMN_NORM {
            return Err(LyapunovError::DegenerateFrame);
        }
        r_diagonal.push(norm);

        // Normalize column i.
        let inverse = 1.0 / norm;
        for value in columns[i].iter_mut() {
            *value *= inverse;
        }

        // Subtract the projection of column i from every later column, so the
        // remaining columns are made orthogonal to the freshly normalized q_i.
        for j in (i + 1)..n {
            let projection = dot(&columns[i], &columns[j]);
            for row in 0..columns[i].len() {
                columns[j][row] -= projection * columns[i][row];
            }
        }
    }

    Ok(r_diagonal)
}

/// Euclidean norm of a vector, accumulated in a fixed order.
fn euclidean_norm(vector: &[f64]) -> Result<f64, LyapunovError> {
    let mut sum = 0.0;
    for &value in vector {
        if !value.is_finite() {
            return Err(LyapunovError::NonFiniteState);
        }
        sum += value * value;
    }
    Ok(sum.sqrt())
}

/// Dot product of two equal-length vectors, accumulated in a fixed order.
fn dot(left: &[f64], right: &[f64]) -> f64 {
    let mut sum = 0.0;
    for (a, b) in left.iter().zip(right) {
        sum += a * b;
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-12, "expected {a} ≈ {b}");
    }

    #[test]
    fn identity_frame_is_already_orthonormal() {
        let mut columns = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let r = gram_schmidt_qr(&mut columns).unwrap();
        approx(r[0], 1.0);
        approx(r[1], 1.0);
        assert_eq!(columns, vec![vec![1.0, 0.0], vec![0.0, 1.0]]);
    }

    #[test]
    fn orthonormalizes_and_reports_column_norms() {
        // Column 0 has length 2 along x; column 1 = (3, 4) has a component along
        // x (removed) and a residual of length 4 along y.
        let mut columns = vec![vec![2.0, 0.0], vec![3.0, 4.0]];
        let r = gram_schmidt_qr(&mut columns).unwrap();
        approx(r[0], 2.0);
        approx(r[1], 4.0);
        // Resulting columns are orthonormal.
        approx(dot(&columns[0], &columns[0]), 1.0);
        approx(dot(&columns[1], &columns[1]), 1.0);
        approx(dot(&columns[0], &columns[1]), 0.0);
    }

    #[test]
    fn preserves_leading_subspace_direction() {
        // The first orthonormal column must point along the first input column.
        let mut columns = vec![vec![0.0, 5.0, 0.0], vec![1.0, 1.0, 0.0], vec![1.0, 1.0, 1.0]];
        let r = gram_schmidt_qr(&mut columns).unwrap();
        approx(r[0], 5.0);
        approx(columns[0][0], 0.0);
        approx(columns[0][1], 1.0);
        approx(columns[0][2], 0.0);
        // Pairwise orthonormal.
        for a in 0..3 {
            for b in 0..3 {
                let expected = if a == b { 1.0 } else { 0.0 };
                approx(dot(&columns[a], &columns[b]), expected);
            }
        }
    }

    #[test]
    fn rejects_a_collapsed_column() {
        let mut columns = vec![vec![1.0, 0.0], vec![0.0, 0.0]];
        assert!(matches!(gram_schmidt_qr(&mut columns), Err(LyapunovError::DegenerateFrame)));
    }

    #[test]
    fn rejects_a_non_finite_entry() {
        let mut columns = vec![vec![f64::NAN, 0.0], vec![0.0, 1.0]];
        assert!(matches!(gram_schmidt_qr(&mut columns), Err(LyapunovError::NonFiniteState)));
    }
}
