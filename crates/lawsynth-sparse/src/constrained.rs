use crate::{RegressionProblem, SparseError, SparseSolution, stlsq::residual_sum_squares};

/// Controls for nonnegative coordinate-descent least squares.
#[derive(Clone, Debug, PartialEq)]
pub struct NonnegativeConfig {
    pub ridge: f64,
    pub max_iterations: usize,
    pub tolerance: f64,
}

impl Default for NonnegativeConfig {
    fn default() -> Self {
        Self {
            ridge: 1e-10,
            max_iterations: 1_000,
            tolerance: 1e-9,
        }
    }
}

/// Fits nonnegative coefficients using cyclic coordinate descent.
///
/// Projection occurs on every coordinate update, keeping every intermediate
/// solution physically admissible for callers that require positive rates.
pub fn nonnegative_least_squares(
    problem: &RegressionProblem,
    config: &NonnegativeConfig,
) -> Result<SparseSolution, SparseError> {
    if !config.ridge.is_finite()
        || config.ridge < 0.0
        || config.max_iterations == 0
        || !config.tolerance.is_finite()
        || config.tolerance <= 0.0
    {
        return Err(SparseError::InvalidConfig);
    }
    let features = problem.features();
    let mut coefficients = vec![0.0; features];
    let mut residual = problem.target.clone();
    let norms = (0..features)
        .map(|feature| {
            problem
                .rows
                .iter()
                .map(|row| row[feature] * row[feature])
                .sum::<f64>()
                + config.ridge
        })
        .collect::<Vec<_>>();
    for _ in 0..config.max_iterations {
        let mut greatest_change: f64 = 0.0;
        for feature in 0..features {
            if norms[feature] <= 1e-14 {
                continue;
            }
            let previous = coefficients[feature];
            let correlation = problem
                .rows
                .iter()
                .zip(&residual)
                .map(|(row, residual)| row[feature] * residual)
                .sum::<f64>()
                + (norms[feature] - config.ridge) * previous;
            let next = (correlation / norms[feature]).max(0.0);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projects_negative_unconstrained_coefficient_to_zero() {
        let problem =
            RegressionProblem::new(vec![vec![1.0, 0.0], vec![0.0, 1.0]], vec![-2.0, 3.0]).unwrap();
        let solution = nonnegative_least_squares(&problem, &NonnegativeConfig::default()).unwrap();
        assert_eq!(solution.coefficients[0], 0.0);
        assert!((solution.coefficients[1] - 3.0).abs() < 1e-8);
    }
}
