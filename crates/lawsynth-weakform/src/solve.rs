use crate::WeakError;

/// Configuration for the deterministic sparse least-squares solve.
#[derive(Clone, Debug, PartialEq)]
pub struct StlsqConfig {
    /// Coefficients with magnitude below this are pruned between refits.
    pub threshold: f64,
    /// Tikhonov (ridge) term added to the normal-equations diagonal.
    pub ridge: f64,
    /// Maximum number of thresholding iterations.
    pub max_iterations: usize,
}

impl Default for StlsqConfig {
    fn default() -> Self {
        Self { threshold: 0.05, ridge: 1e-8, max_iterations: 12 }
    }
}

/// A fitted coefficient vector and its residual norm for one target.
#[derive(Clone, Debug, PartialEq)]
pub struct StlsqFit {
    /// Coefficients over the full candidate library (pruned entries are `0`).
    pub coefficients: Vec<f64>,
    /// Euclidean residual `‖rows · coefficients − target‖₂`.
    pub residual: f64,
}

/// Sequentially-thresholded least squares over a (possibly over-determined)
/// linear system `rows · coefficients ≈ target`.
///
/// This is the same algorithm strong-form SINDy uses, applied here to the weak
/// system `G Ξ = B`. It is deterministic: the least-squares refit solves the
/// ridge-regularised normal equations by Gaussian elimination with partial
/// pivoting (a fixed pivot rule), and thresholding drops columns by a fixed
/// magnitude test. Identical inputs give bit-identical output.
pub fn stlsq(
    rows: &[Vec<f64>],
    target: &[f64],
    config: &StlsqConfig,
) -> Result<StlsqFit, WeakError> {
    if !config.threshold.is_finite() || config.threshold < 0.0 {
        return Err(WeakError::InvalidConfig { field: "threshold" });
    }
    if !config.ridge.is_finite() || config.ridge < 0.0 {
        return Err(WeakError::InvalidConfig { field: "ridge" });
    }
    if config.max_iterations == 0 {
        return Err(WeakError::InvalidConfig { field: "max_iterations" });
    }
    let width = rows.first().map_or(0, Vec::len);
    let mut active: Vec<usize> = (0..width).collect();
    let mut coefficients = vec![0.0; width];

    for _ in 0..config.max_iterations {
        if active.is_empty() {
            break;
        }
        let fitted = solve_active(rows, target, &active, config.ridge)?;
        coefficients.fill(0.0);
        for (&column, value) in active.iter().zip(fitted) {
            coefficients[column] = value;
        }
        let next: Vec<usize> = active
            .iter()
            .copied()
            .filter(|&column| coefficients[column].abs() >= config.threshold)
            .collect();
        if next == active {
            break;
        }
        active = next;
    }

    Ok(StlsqFit { residual: residual_norm(rows, target, &coefficients), coefficients })
}

/// A dimensionless conditioning proxy for the full weak library: the ratio of
/// the largest to the smallest pivot magnitude encountered while factoring the
/// ridge-regularised normal matrix `GᵀG + ridge·I`.
///
/// This is a cheap, deterministic health signal, not a true 2-norm condition
/// number (which would require an SVD). A large value warns that the placed test
/// functions do not excite the candidate columns independently. Returns
/// `f64::INFINITY` if the normal matrix is singular.
pub(crate) fn conditioning(rows: &[Vec<f64>], ridge: f64) -> f64 {
    let width = rows.first().map_or(0, Vec::len);
    if width == 0 {
        return f64::INFINITY;
    }
    let active: Vec<usize> = (0..width).collect();
    let matrix = normal_matrix(rows, &active, ridge);
    match lu_pivots(matrix) {
        Some(pivots) => {
            let max = pivots.iter().cloned().fold(0.0_f64, f64::max);
            let min = pivots.iter().cloned().fold(f64::INFINITY, f64::min);
            if min <= 0.0 { f64::INFINITY } else { max / min }
        }
        None => f64::INFINITY,
    }
}

fn residual_norm(rows: &[Vec<f64>], target: &[f64], coefficients: &[f64]) -> f64 {
    rows.iter()
        .zip(target)
        .map(|(row, &observed)| {
            let predicted: f64 = row.iter().zip(coefficients).map(|(x, w)| x * w).sum();
            let residual = predicted - observed;
            residual * residual
        })
        .sum::<f64>()
        .sqrt()
}

fn normal_matrix(rows: &[Vec<f64>], active: &[usize], ridge: f64) -> Vec<Vec<f64>> {
    let width = active.len();
    let mut matrix = vec![vec![0.0; width]; width];
    for row in rows {
        for (left_pos, &left) in active.iter().enumerate() {
            for (right_pos, &right) in active.iter().enumerate() {
                matrix[left_pos][right_pos] += row[left] * row[right];
            }
        }
    }
    for (index, diagonal) in matrix.iter_mut().enumerate() {
        diagonal[index] += ridge;
    }
    matrix
}

fn solve_active(
    rows: &[Vec<f64>],
    target: &[f64],
    active: &[usize],
    ridge: f64,
) -> Result<Vec<f64>, WeakError> {
    let matrix = normal_matrix(rows, active, ridge);
    let mut rhs = vec![0.0; active.len()];
    for (row, &observed) in rows.iter().zip(target) {
        for (position, &column) in active.iter().enumerate() {
            rhs[position] += row[column] * observed;
        }
    }
    gaussian_solve(matrix, rhs)
}

/// Solves `matrix · x = rhs` by Gaussian elimination with partial pivoting.
fn gaussian_solve(mut matrix: Vec<Vec<f64>>, mut rhs: Vec<f64>) -> Result<Vec<f64>, WeakError> {
    let size = rhs.len();
    for pivot in 0..size {
        let best = (pivot..size)
            .max_by(|&left, &right| {
                matrix[left][pivot].abs().total_cmp(&matrix[right][pivot].abs())
            })
            .expect("non-empty pivot range");
        if matrix[best][pivot].abs() < 1e-14 {
            return Err(WeakError::SingularSystem);
        }
        matrix.swap(pivot, best);
        rhs.swap(pivot, best);
        let scale = matrix[pivot][pivot];
        for value in matrix[pivot].iter_mut().skip(pivot) {
            *value /= scale;
        }
        rhs[pivot] /= scale;
        let pivot_row = matrix[pivot].clone();
        for row in 0..size {
            if row == pivot {
                continue;
            }
            let factor = matrix[row][pivot];
            for (column, value) in matrix[row].iter_mut().enumerate().skip(pivot) {
                *value -= factor * pivot_row[column];
            }
            rhs[row] -= factor * rhs[pivot];
        }
    }
    Ok(rhs)
}

/// Factors `matrix` with partial pivoting and returns the pivot magnitudes, or
/// `None` if a pivot is numerically zero (singular).
fn lu_pivots(mut matrix: Vec<Vec<f64>>) -> Option<Vec<f64>> {
    let size = matrix.len();
    let mut pivots = Vec::with_capacity(size);
    for pivot in 0..size {
        let best = (pivot..size).max_by(|&left, &right| {
            matrix[left][pivot].abs().total_cmp(&matrix[right][pivot].abs())
        })?;
        let magnitude = matrix[best][pivot].abs();
        if magnitude < 1e-300 {
            return None;
        }
        matrix.swap(pivot, best);
        pivots.push(magnitude);
        let pivot_row = matrix[pivot].clone();
        for target_row in matrix.iter_mut().skip(pivot + 1) {
            let factor = target_row[pivot] / pivot_row[pivot];
            for (column, value) in target_row.iter_mut().enumerate().skip(pivot) {
                *value -= factor * pivot_row[column];
            }
        }
    }
    Some(pivots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_a_sparse_solution_and_prunes_noise_columns() {
        // target = 2 * column1, with an unrelated column0 that must be pruned.
        let rows =
            vec![vec![1.0, 0.0], vec![1.0, 1.0], vec![1.0, 2.0], vec![1.0, 3.0], vec![1.0, 4.0]];
        let target = vec![0.0, 2.0, 4.0, 6.0, 8.0];
        let fit =
            stlsq(&rows, &target, &StlsqConfig { threshold: 0.1, ..Default::default() }).unwrap();
        assert!(fit.coefficients[0].abs() < 1e-8);
        assert!((fit.coefficients[1] - 2.0).abs() < 1e-8);
        assert!(fit.residual < 1e-8);
    }

    #[test]
    fn solves_an_overdetermined_system_in_least_squares() {
        // Five equations, one unknown slope through the origin: y = 3 x.
        let rows: Vec<Vec<f64>> = (1..=5).map(|i| vec![i as f64]).collect();
        let target: Vec<f64> = (1..=5).map(|i| 3.0 * i as f64).collect();
        let fit = stlsq(&rows, &target, &StlsqConfig::default()).unwrap();
        assert!((fit.coefficients[0] - 3.0).abs() < 1e-9);
    }

    #[test]
    fn conditioning_is_finite_for_a_well_posed_library() {
        let rows = vec![vec![1.0, 0.0], vec![0.0, 1.0], vec![1.0, 1.0]];
        let value = conditioning(&rows, 0.0);
        assert!(value.is_finite() && value >= 1.0);
    }

    #[test]
    fn rejects_a_negative_threshold() {
        let rows = vec![vec![1.0]];
        let target = vec![1.0];
        let result = stlsq(&rows, &target, &StlsqConfig { threshold: -1.0, ..Default::default() });
        assert!(matches!(result, Err(WeakError::InvalidConfig { field: "threshold" })));
    }
}
