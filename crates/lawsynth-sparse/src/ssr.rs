use crate::{
    RegressionProblem, SparseConfig, SparseError, SparseSolution,
    stlsq::{residual_sum_squares, solve_active, validate_config},
};

/// One model along the stepwise elimination path.
#[derive(Clone, Debug, PartialEq)]
pub struct SsrStep {
    /// Retained column indices at this path point, in ascending order.
    pub support: Vec<usize>,
    /// Residual sum of squares of the least-squares refit on `support`.
    pub residual_sum_squares: f64,
    /// Akaike information criterion (AIC) used to select along the path.
    pub akaike: f64,
}

/// Stepwise Sparse Regression (SSR).
///
/// Starting from the full least-squares fit, the term with the smallest absolute
/// coefficient is removed, the model is refit on the survivors, and the process
/// repeats down to a single term. Each support size along that path is scored
/// with the Akaike information criterion (AIC), and the support minimising AIC is
/// returned (ties resolve to the sparsest model, then to the lowest indices). The
/// procedure reads no clock and draws no random numbers, so it is fully
/// deterministic.
pub fn ssr(
    problem: &RegressionProblem,
    config: &SparseConfig,
) -> Result<SparseSolution, SparseError> {
    validate_config(config)?;
    let path = elimination_path(problem, config)?;
    let features = problem.features();
    // Select the AIC-minimising model; on ties prefer fewer terms so the sparser
    // structure wins, keeping the choice deterministic.
    let best = path
        .iter()
        .min_by(|left, right| {
            left.akaike.total_cmp(&right.akaike).then(left.support.len().cmp(&right.support.len()))
        })
        .ok_or(SparseError::EmptyProblem)?;
    let mut coefficients = vec![0.0; features];
    let fitted = solve_active(problem, &best.support, config.ridge, None)?;
    for (index, value) in best.support.iter().zip(fitted) {
        coefficients[*index] = value;
    }
    Ok(SparseSolution {
        residual_sum_squares: residual_sum_squares(problem, &coefficients),
        coefficients,
    })
}

/// Builds the full elimination path from the dense fit down to one term. Exposed
/// to the crate so tests can assert on the path shape and AIC ordering.
pub(crate) fn elimination_path(
    problem: &RegressionProblem,
    config: &SparseConfig,
) -> Result<Vec<SsrStep>, SparseError> {
    let features = problem.features();
    let observations = problem.rows.len();
    let mut active = (0..features).collect::<Vec<_>>();
    let mut path = Vec::with_capacity(features);
    loop {
        let fitted = solve_active(problem, &active, config.ridge, None)?;
        let mut coefficients = vec![0.0; features];
        for (index, value) in active.iter().zip(&fitted) {
            coefficients[*index] = *value;
        }
        let residual_sum_squares = residual_sum_squares(problem, &coefficients);
        path.push(SsrStep {
            support: active.clone(),
            residual_sum_squares,
            akaike: akaike(residual_sum_squares, observations, active.len()),
        });
        if active.len() == 1 {
            break;
        }
        active.remove(least_important(&fitted));
    }
    Ok(path)
}

/// Position within `coefficients` of the smallest-magnitude term. Ties resolve to
/// the lowest position, so elimination is order-independent and reproducible.
fn least_important(coefficients: &[f64]) -> usize {
    let mut best = 0;
    let mut best_magnitude = coefficients[0].abs();
    for (position, value) in coefficients.iter().enumerate().skip(1) {
        let magnitude = value.abs();
        if magnitude < best_magnitude {
            best_magnitude = magnitude;
            best = position;
        }
    }
    best
}

/// Gaussian-likelihood Akaike information criterion, `n·ln(RSS/n) + 2k`. The
/// residual energy is floored to keep the logarithm finite when a support fits
/// the data exactly, so a perfect fit still yields a finite, comparable score.
fn akaike(residual_sum_squares: f64, observations: usize, terms: usize) -> f64 {
    let residual = residual_sum_squares.max(1e-300) / observations as f64;
    observations as f64 * residual.ln() + 2.0 * terms as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sparse_problem() -> RegressionProblem {
        // y = 2*x0 - 1*x2 exactly; x1 and x3 are spurious independent columns.
        let rows = vec![
            vec![1.0, 0.5, 2.0, 1.0],
            vec![2.0, 1.5, 1.0, 0.0],
            vec![0.0, 2.0, 3.0, 2.5],
            vec![3.0, 1.0, 0.5, 1.5],
            vec![1.0, 3.0, 2.0, 0.5],
            vec![2.5, 0.0, 1.5, 3.0],
        ];
        let target = rows.iter().map(|row| 2.0 * row[0] - 1.0 * row[2]).collect::<Vec<_>>();
        RegressionProblem::new(rows, target).unwrap()
    }

    #[test]
    fn path_shrinks_one_term_at_a_time_to_a_single_term() {
        let problem = sparse_problem();
        let path = elimination_path(&problem, &SparseConfig::default()).unwrap();
        assert_eq!(path.len(), 4);
        assert_eq!(path[0].support.len(), 4);
        assert_eq!(path[3].support.len(), 1);
        for window in path.windows(2) {
            assert_eq!(window[0].support.len(), window[1].support.len() + 1);
        }
    }

    #[test]
    fn selects_the_true_two_term_support() {
        let problem = sparse_problem();
        let solution = ssr(&problem, &SparseConfig::default()).unwrap();
        assert!((solution.coefficients[0] - 2.0).abs() < 1e-6);
        assert_eq!(solution.coefficients[1], 0.0);
        assert!((solution.coefficients[2] + 1.0).abs() < 1e-6);
        assert_eq!(solution.coefficients[3], 0.0);
        assert!(solution.residual_sum_squares < 1e-12);
    }

    #[test]
    fn is_deterministic() {
        let problem = sparse_problem();
        let first = ssr(&problem, &SparseConfig::default()).unwrap();
        let second = ssr(&problem, &SparseConfig::default()).unwrap();
        assert_eq!(first, second);
    }
}
