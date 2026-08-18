use crate::{
    RegressionProblem, SparseConfig, SparseError, SparseSolution,
    stlsq::{residual_sum_squares, solve_active, validate_config},
};

/// Sparse relaxed regularized regression (SR3) using hard thresholding.
pub fn sr3(
    problem: &RegressionProblem,
    config: &SparseConfig,
) -> Result<SparseSolution, SparseError> {
    validate_config(config)?;
    let active = (0..problem.features()).collect::<Vec<_>>();
    let penalty = config.ridge.max(1e-6);
    let mut relaxed = vec![0.0; problem.features()];
    for _ in 0..config.max_iterations {
        let next = solve_active(problem, &active, config.ridge, Some((&relaxed, penalty)))?;
        let mut changed = false;
        for (index, value) in next.into_iter().enumerate() {
            let thresholded = if value.abs() >= config.threshold { value } else { 0.0 };
            changed |= (relaxed[index] - thresholded).abs() > 1e-12;
            relaxed[index] = thresholded;
        }
        if !changed {
            break;
        }
    }
    Ok(SparseSolution {
        residual_sum_squares: residual_sum_squares(problem, &relaxed),
        coefficients: relaxed,
    })
}

#[cfg(test)]
mod tests {
    use crate::{RegressionProblem, SparseConfig};

    use super::sr3;

    #[test]
    fn shrinks_irrelevant_features() {
        let problem = RegressionProblem::new(
            vec![vec![0.0, 1.0], vec![1.0, 1.0], vec![2.0, 1.0]],
            vec![0.0, 3.0, 6.0],
        )
        .unwrap();
        let result = sr3(&problem, &SparseConfig { threshold: 0.1, ..Default::default() }).unwrap();
        assert!((result.coefficients[0] - 3.0).abs() < 1e-4);
    }
}
