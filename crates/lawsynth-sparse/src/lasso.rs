use crate::{RegressionProblem, SparseError, SparseSolution, stlsq::residual_sum_squares};

/// Coordinate-descent controls for L1-regularized least squares.
#[derive(Clone, Debug, PartialEq)]
pub struct LassoConfig {
    pub lambda: f64,
    pub max_iterations: usize,
    pub tolerance: f64,
}

impl Default for LassoConfig {
    fn default() -> Self {
        Self {
            lambda: 0.05,
            max_iterations: 1_000,
            tolerance: 1e-9,
        }
    }
}

/// Fits a LASSO model with cyclic coordinate descent and soft thresholding.
pub fn lasso(
    problem: &RegressionProblem,
    config: &LassoConfig,
) -> Result<SparseSolution, SparseError> {
    if !config.lambda.is_finite()
        || config.lambda < 0.0
        || config.max_iterations == 0
        || !config.tolerance.is_finite()
        || config.tolerance <= 0.0
    {
        return Err(SparseError::InvalidConfig);
    }
    let features = problem.features();
    let mut coefficients = vec![0.0; features];
    let mut residual = problem.target.clone();
    let squared_norms = (0..features)
        .map(|feature| {
            problem
                .rows
                .iter()
                .map(|row| row[feature] * row[feature])
                .sum::<f64>()
        })
        .collect::<Vec<_>>();

    for _ in 0..config.max_iterations {
        let mut greatest_change: f64 = 0.0;
        for feature in 0..features {
            if squared_norms[feature] <= 1e-14 {
                continue;
            }
            let previous = coefficients[feature];
            let correlation = problem
                .rows
                .iter()
                .zip(&residual)
                .map(|(row, residual)| row[feature] * residual)
                .sum::<f64>()
                + squared_norms[feature] * previous;
            let next = soft_threshold(correlation, config.lambda) / squared_norms[feature];
            let delta = next - previous;
            if delta != 0.0 {
                for (row, residual) in problem.rows.iter().zip(&mut residual) {
                    *residual -= row[feature] * delta;
                }
                coefficients[feature] = next;
                greatest_change = greatest_change.max(delta.abs());
            }
        }
        if greatest_change <= config.tolerance {
            break;
        }
    }
    Ok(SparseSolution {
        residual_sum_squares: residual_sum_squares(problem, &coefficients),
        coefficients,
    })
}

fn soft_threshold(value: f64, lambda: f64) -> f64 {
    if value > lambda {
        value - lambda
    } else if value < -lambda {
        value + lambda
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lasso_removes_a_weak_irrelevant_feature() {
        let problem = RegressionProblem::new(
            vec![
                vec![0.0, 1.0],
                vec![1.0, 1.0],
                vec![2.0, 1.0],
                vec![3.0, 1.0],
            ],
            vec![0.0, 2.0, 4.0, 6.0],
        )
        .unwrap();
        let solution = lasso(
            &problem,
            &LassoConfig {
                lambda: 0.1,
                ..Default::default()
            },
        )
        .unwrap();
        assert!((solution.coefficients[0] - 1.992_857).abs() < 1e-5);
        assert!(solution.coefficients[1].abs() < 1e-6);
    }
}
