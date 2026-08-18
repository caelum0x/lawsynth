use crate::{RegressionProblem, SparseConfig, SparseError};

/// Coefficients fitted against the complete original feature matrix.
#[derive(Clone, Debug, PartialEq)]
pub struct SparseSolution {
    pub coefficients: Vec<f64>,
    pub residual_sum_squares: f64,
}

/// Sequentially thresholded least squares with deterministic pivoting.
pub fn stlsq(
    problem: &RegressionProblem,
    config: &SparseConfig,
) -> Result<SparseSolution, SparseError> {
    validate_config(config)?;
    let mut active = (0..problem.features()).collect::<Vec<_>>();
    let mut coefficients = vec![0.0; problem.features()];
    for _ in 0..config.max_iterations {
        if active.is_empty() {
            break;
        }
        let fitted = solve_active(problem, &active, config.ridge, None)?;
        coefficients.fill(0.0);
        for (index, coefficient) in active.iter().zip(fitted) {
            coefficients[*index] = coefficient;
        }
        let next = active
            .iter()
            .copied()
            .filter(|index| coefficients[*index].abs() >= config.threshold)
            .collect::<Vec<_>>();
        if next == active {
            break;
        }
        // A threshold large enough to prune every remaining term drains `active`
        // to empty. The reported support is then empty, so the coefficients must
        // be the all-zero model — not the pre-prune dense fit still sitting in
        // `coefficients`. Zero them before breaking so an aggressive threshold
        // over-prunes (as intended) instead of silently returning a dense law.
        if next.is_empty() {
            coefficients.fill(0.0);
            break;
        }
        active = next;
    }
    Ok(SparseSolution {
        residual_sum_squares: residual_sum_squares(problem, &coefficients),
        coefficients,
    })
}

pub(crate) fn validate_config(config: &SparseConfig) -> Result<(), SparseError> {
    if !config.threshold.is_finite()
        || config.threshold < 0.0
        || !config.ridge.is_finite()
        || config.ridge < 0.0
        || config.max_iterations == 0
    {
        return Err(SparseError::InvalidConfig);
    }
    Ok(())
}

pub(crate) fn solve_active(
    problem: &RegressionProblem,
    active: &[usize],
    ridge: f64,
    prior: Option<(&[f64], f64)>,
) -> Result<Vec<f64>, SparseError> {
    let width = active.len();
    let mut matrix = vec![vec![0.0; width]; width];
    let mut target = vec![0.0; width];
    for (row, observed) in problem.rows.iter().zip(&problem.target) {
        for (left_position, left) in active.iter().enumerate() {
            target[left_position] += row[*left] * observed;
            for (right_position, right) in active.iter().enumerate() {
                matrix[left_position][right_position] += row[*left] * row[*right];
            }
        }
    }
    for (index, row) in matrix.iter_mut().enumerate() {
        row[index] += ridge;
    }
    if let Some((prior_values, penalty)) = prior {
        for (position, index) in active.iter().enumerate() {
            matrix[position][position] += penalty;
            target[position] += penalty * prior_values[*index];
        }
    }
    solve_linear_system(matrix, target)
}

pub(crate) fn residual_sum_squares(problem: &RegressionProblem, coefficients: &[f64]) -> f64 {
    problem
        .rows
        .iter()
        .zip(&problem.target)
        .map(|(row, target)| {
            let residual = row.iter().zip(coefficients).map(|(x, w)| x * w).sum::<f64>() - target;
            residual * residual
        })
        .sum()
}

fn solve_linear_system(
    mut matrix: Vec<Vec<f64>>,
    mut target: Vec<f64>,
) -> Result<Vec<f64>, SparseError> {
    for pivot in 0..target.len() {
        let best = (pivot..target.len())
            .max_by(|left, right| {
                matrix[*left][pivot].abs().total_cmp(&matrix[*right][pivot].abs())
            })
            .expect("non-empty pivot range");
        if matrix[best][pivot].abs() < 1e-14 {
            return Err(SparseError::SingularSystem);
        }
        matrix.swap(pivot, best);
        target.swap(pivot, best);
        let scale = matrix[pivot][pivot];
        for value in matrix[pivot].iter_mut().skip(pivot) {
            *value /= scale;
        }
        target[pivot] /= scale;
        let pivot_row = matrix[pivot].clone();
        for row in 0..target.len() {
            if row == pivot {
                continue;
            }
            let factor = matrix[row][pivot];
            for (column, value) in matrix[row].iter_mut().enumerate().skip(pivot) {
                *value -= factor * pivot_row[column];
            }
            target[row] -= factor * target[pivot];
        }
    }
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_a_sparse_linear_law() {
        let problem = RegressionProblem::new(
            vec![vec![1.0, 0.0], vec![1.0, 1.0], vec![1.0, 2.0], vec![1.0, 3.0]],
            vec![0.0, 2.0, 4.0, 6.0],
        )
        .unwrap();
        let result =
            stlsq(&problem, &SparseConfig { threshold: 0.1, ..Default::default() }).unwrap();
        assert!(result.coefficients[0].abs() < 1e-8);
        assert!((result.coefficients[1] - 2.0).abs() < 1e-8);
    }

    #[test]
    fn threshold_above_every_coefficient_yields_the_empty_model() {
        // The true slope is 2.0; a threshold of 10.0 exceeds every coefficient,
        // so STLSQ must drain to the all-zero model rather than returning the
        // pre-prune dense fit.
        let problem = RegressionProblem::new(
            vec![vec![1.0, 0.0], vec![1.0, 1.0], vec![1.0, 2.0], vec![1.0, 3.0]],
            vec![0.0, 2.0, 4.0, 6.0],
        )
        .unwrap();
        let result =
            stlsq(&problem, &SparseConfig { threshold: 10.0, ..Default::default() }).unwrap();
        assert!(result.coefficients.iter().all(|coefficient| *coefficient == 0.0));
    }
}
