use crate::{
    RegressionProblem, SparseConfig, SparseError, SparseSolution,
    stlsq::{residual_sum_squares, solve_active, validate_config},
};

/// Smallest error-reduction ratio (ERR) worth selecting. Below this the greedy
/// forward search stops: on a clean sparse system every spurious column becomes
/// orthogonal to the target once the true support is selected, so its ERR
/// collapses to numerical zero.
const ERR_FLOOR: f64 = 1e-12;

/// One accepted step of the forward selection, retained for inspection and tests.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrolsStep {
    /// Column index selected at this step, in the caller's feature ordering.
    pub index: usize,
    /// Error-reduction ratio of the column when it was selected.
    pub error_reduction_ratio: f64,
}

/// Forward Regression with Orthogonal Least Squares (FROLS).
///
/// At each step the candidate column with the largest error-reduction ratio
/// (ERR) is selected, the remaining columns are orthogonalised against it with
/// modified Gram-Schmidt, and the search repeats until the term-count cap
/// ([`SparseConfig::max_iterations`]) is reached or no remaining column clears
/// [`ERR_FLOOR`]. Final coefficients are an ordinary least-squares refit of the
/// selected support in the caller's original feature basis, so unselected
/// columns are returned as exact zeros. The procedure reads no clock and draws
/// no random numbers, so it is fully deterministic; ties in ERR resolve to the
/// lowest column index.
pub fn frols(
    problem: &RegressionProblem,
    config: &SparseConfig,
) -> Result<SparseSolution, SparseError> {
    validate_config(config)?;
    let features = problem.features();
    let mut coefficients = vec![0.0; features];
    let support = greedy_err_support(problem, config)?;
    if !support.is_empty() {
        let active = support.iter().map(|step| step.index).collect::<Vec<_>>();
        let fitted = solve_active(problem, &active, config.ridge, None)?;
        for (index, value) in active.iter().zip(fitted) {
            coefficients[*index] = value;
        }
    }
    Ok(SparseSolution {
        residual_sum_squares: residual_sum_squares(problem, &coefficients),
        coefficients,
    })
}

/// Greedy ERR-ordered forward selection, exposed to the crate so both [`frols`]
/// and its tests can inspect the exact order the true terms are recovered in.
pub(crate) fn greedy_err_support(
    problem: &RegressionProblem,
    config: &SparseConfig,
) -> Result<Vec<FrolsStep>, SparseError> {
    let features = problem.features();
    let target = &problem.target;
    let target_energy = dot(target, target);
    if target_energy <= f64::MIN_POSITIVE {
        return Ok(Vec::new());
    }
    // Working orthogonalised copy of every candidate column (modified Gram-Schmidt).
    let mut columns =
        (0..features).map(|column| extract_column(problem, column)).collect::<Vec<_>>();
    let mut selected = vec![false; features];
    let mut support = Vec::new();
    let max_terms = config.max_iterations.min(features);
    while support.len() < max_terms {
        let mut best: Option<FrolsStep> = None;
        for (index, orthogonal) in columns.iter().enumerate() {
            if selected[index] {
                continue;
            }
            let energy = dot(orthogonal, orthogonal);
            if energy <= 1e-14 {
                // Degenerate or fully collinear with the selected basis.
                continue;
            }
            let projection = dot(orthogonal, target);
            let error_reduction_ratio = (projection * projection) / (energy * target_energy);
            // Strictly-greater keeps the lowest index on ties, so selection is
            // order-independent and reproducible.
            if best.is_none_or(|current| error_reduction_ratio > current.error_reduction_ratio) {
                best = Some(FrolsStep { index, error_reduction_ratio });
            }
        }
        let Some(step) = best else {
            break;
        };
        if step.error_reduction_ratio < ERR_FLOOR {
            break;
        }
        // Orthogonalise every remaining column against the newly selected one.
        let basis = columns[step.index].clone();
        let basis_energy = dot(&basis, &basis);
        for (index, column) in columns.iter_mut().enumerate() {
            if selected[index] || index == step.index {
                continue;
            }
            let projection = dot(&basis, column) / basis_energy;
            for (value, base) in column.iter_mut().zip(&basis) {
                *value -= projection * base;
            }
        }
        selected[step.index] = true;
        support.push(step);
    }
    Ok(support)
}

fn extract_column(problem: &RegressionProblem, column: usize) -> Vec<f64> {
    problem.rows.iter().map(|row| row[column]).collect()
}

fn dot(left: &[f64], right: &[f64]) -> f64 {
    left.iter().zip(right).map(|(a, b)| a * b).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clean_problem() -> RegressionProblem {
        // Target y = 3*x1 + 0*x2 - 2*x3 with a spurious x2 column. Columns are
        // linearly independent so the true support is exactly {0, 2}.
        let rows = vec![
            vec![1.0, 2.0, 0.5],
            vec![2.0, 1.0, 1.0],
            vec![3.0, 0.0, 2.0],
            vec![0.5, 4.0, 1.5],
            vec![1.5, 3.0, 0.0],
        ];
        let target =
            rows.iter().map(|row| 3.0 * row[0] + 0.0 * row[1] - 2.0 * row[2]).collect::<Vec<_>>();
        RegressionProblem::new(rows, target).unwrap()
    }

    #[test]
    fn err_ordering_selects_the_true_terms_first() {
        let problem = clean_problem();
        let support = greedy_err_support(
            &problem,
            &SparseConfig { threshold: 0.1, max_iterations: 3, ridge: 1e-10 },
        )
        .unwrap();
        // Only the two true terms clear the ERR floor; the spurious column is
        // orthogonal to the target after they are selected.
        assert_eq!(support.len(), 2);
        let selected = support.iter().map(|step| step.index).collect::<Vec<_>>();
        assert!(selected.contains(&0));
        assert!(selected.contains(&2));
        assert!(!selected.contains(&1));
        // ERR values are recorded in descending selection order.
        assert!(support[0].error_reduction_ratio >= support[1].error_reduction_ratio);
    }

    #[test]
    fn recovers_exact_sparse_coefficients() {
        let problem = clean_problem();
        let solution =
            frols(&problem, &SparseConfig { threshold: 0.1, max_iterations: 5, ridge: 1e-12 })
                .unwrap();
        assert!((solution.coefficients[0] - 3.0).abs() < 1e-8);
        assert_eq!(solution.coefficients[1], 0.0);
        assert!((solution.coefficients[2] + 2.0).abs() < 1e-8);
        assert!(solution.residual_sum_squares < 1e-16);
    }

    #[test]
    fn is_deterministic() {
        let problem = clean_problem();
        let config = SparseConfig { threshold: 0.1, max_iterations: 4, ridge: 1e-10 };
        let first = frols(&problem, &config).unwrap();
        let second = frols(&problem, &config).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn term_cap_limits_the_support() {
        let problem = clean_problem();
        let support = greedy_err_support(
            &problem,
            &SparseConfig { threshold: 0.1, max_iterations: 1, ridge: 1e-10 },
        )
        .unwrap();
        assert_eq!(support.len(), 1);
    }
}
