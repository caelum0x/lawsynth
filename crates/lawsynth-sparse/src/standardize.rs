use crate::stlsq::residual_sum_squares;
use crate::{RegressionProblem, SparseError, SparseSolution};

/// Root-mean-square scale for every feature column.
#[derive(Clone, Debug, PartialEq)]
pub struct FeatureScaling {
    pub scales: Vec<f64>,
}

/// Scales each feature to unit root-mean-square magnitude.
///
/// Unlike mean-centering, RMS scaling preserves the meaning of a constant
/// feature as an intercept and allows a fitted law to be mapped directly back
/// to the original expression basis.
pub fn standardize_problem(
    problem: &RegressionProblem,
) -> Result<(RegressionProblem, FeatureScaling), SparseError> {
    let observations = problem.rows.len() as f64;
    let scales = (0..problem.features())
        .map(|column| {
            let sum_squares = problem
                .rows
                .iter()
                .map(|row| row[column] * row[column])
                .sum::<f64>();
            let scale = (sum_squares / observations).sqrt();
            if scale > 1e-14 { scale } else { 1.0 }
        })
        .collect::<Vec<_>>();
    let rows = problem
        .rows
        .iter()
        .map(|row| {
            row.iter()
                .zip(&scales)
                .map(|(value, scale)| value / scale)
                .collect()
        })
        .collect();
    Ok((
        RegressionProblem::new(rows, problem.target.clone())?,
        FeatureScaling { scales },
    ))
}

pub(crate) fn restore_solution(
    original: &RegressionProblem,
    mut solution: SparseSolution,
    scaling: &FeatureScaling,
) -> Result<SparseSolution, SparseError> {
    if solution.coefficients.len() != scaling.scales.len()
        || solution.coefficients.len() != original.features()
    {
        return Err(SparseError::RowLengthMismatch);
    }
    for (coefficient, scale) in solution.coefficients.iter_mut().zip(&scaling.scales) {
        *coefficient /= scale;
    }
    solution.residual_sum_squares = residual_sum_squares(original, &solution.coefficients);
    Ok(solution)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SparseConfig, stlsq_standardized};

    #[test]
    fn scaling_round_trips_coefficients_in_original_units() {
        let problem = RegressionProblem::new(
            vec![
                vec![1.0, 1_000.0],
                vec![1.0, 2_000.0],
                vec![1.0, 3_000.0],
                vec![1.0, 4_000.0],
            ],
            vec![3.0, 5.0, 7.0, 9.0],
        )
        .unwrap();
        let solution = stlsq_standardized(
            &problem,
            &SparseConfig {
                threshold: 1e-8,
                ..Default::default()
            },
        )
        .unwrap();
        assert!((solution.coefficients[0] - 1.0).abs() < 1e-8);
        assert!((solution.coefficients[1] - 0.002).abs() < 1e-10);
        assert!(solution.residual_sum_squares < 1e-16);
    }

    #[test]
    fn zero_columns_remain_finite() {
        let problem = RegressionProblem::new(
            vec![vec![0.0, 1.0], vec![0.0, 2.0], vec![0.0, 3.0]],
            vec![2.0, 4.0, 6.0],
        )
        .unwrap();
        let (scaled, scaling) = standardize_problem(&problem).unwrap();
        assert_eq!(scaling.scales[0], 1.0);
        assert!(scaled.rows.iter().all(|row| row[0] == 0.0));
    }
}
