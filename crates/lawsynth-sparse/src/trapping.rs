use crate::{
    RegressionProblem, SparseConfig, SparseError, SparseSolution,
    stlsq::{residual_sum_squares, solve_active, validate_config},
};

/// Controls for the stability-penalised SR3 ("trapping") optimiser.
///
/// The bias targets a single linear self-feedback ("diagonal") term — the column
/// whose coefficient is the diagonal entry `A_ii` of a quadratic system's linear
/// operator. For a scalar equation the symmetric part of `A_ii` is the
/// coefficient itself, so a negative value promotes boundedness. When
/// [`diagonal`](Self::diagonal) is `None` or [`stability_weight`](Self::stability_weight)
/// is `0.0`, the optimiser reduces exactly to [`crate::sr3`].
#[derive(Clone, Debug, PartialEq)]
pub struct TrappingConfig {
    /// Thresholding, iteration, and ridge controls shared with SR3.
    pub sparse: SparseConfig,
    /// Column index of the linear self-feedback term whose positive part is
    /// damped toward the stable (negative) half-space.
    pub diagonal: Option<usize>,
    /// Strength of the one-sided damping applied to a positive self-feedback
    /// coefficient. Larger values pull it more firmly toward zero.
    pub stability_weight: f64,
}

impl Default for TrappingConfig {
    fn default() -> Self {
        Self { sparse: SparseConfig::default(), diagonal: None, stability_weight: 1.0 }
    }
}

/// Stability-biased sparse relaxed regression (trapping-flavoured SR3).
///
/// This runs the same relaxed-variable SR3 iteration as [`crate::sr3`], but after
/// each least-squares solve it damps a *positive* self-feedback coefficient
/// toward the negative half-space with a deterministic one-sided projection
/// (`v ↦ v / (1 + weight)` when `v > 0`). Biasing the diagonal linear term
/// negative encourages a negative-(semi)definite symmetric linear part, which is
/// the structural precondition the trapping theorem uses to certify global
/// boundedness of quadratic ODE systems.
///
/// Honesty note: this is a stability *bias*, not a global-boundedness *proof*. It
/// does not solve the trapping semidefinite program, so it cannot guarantee a
/// trapping region exists. It nudges the fit toward the stable regime while
/// leaving every non-diagonal coefficient governed by ordinary SR3. It is fully
/// deterministic: no clock is read and no random numbers are drawn.
pub fn trapping(
    problem: &RegressionProblem,
    config: &TrappingConfig,
) -> Result<SparseSolution, SparseError> {
    validate_config(&config.sparse)?;
    if !config.stability_weight.is_finite() || config.stability_weight < 0.0 {
        return Err(SparseError::InvalidConfig);
    }
    if let Some(index) = config.diagonal {
        if index >= problem.features() {
            return Err(SparseError::InvalidConfig);
        }
    }
    let features = problem.features();
    let active = (0..features).collect::<Vec<_>>();
    let penalty = config.sparse.ridge.max(1e-6);
    let mut relaxed = vec![0.0; features];
    for _ in 0..config.sparse.max_iterations {
        let next = solve_active(problem, &active, config.sparse.ridge, Some((&relaxed, penalty)))?;
        let mut changed = false;
        for (index, value) in next.into_iter().enumerate() {
            // Stability bias: damp a positive self-feedback coefficient toward the
            // bounded (negative) half-space before thresholding. The damped value
            // becomes the relaxed prior for the next solve, so the bias propagates
            // to convergence instead of being applied only once.
            let biased = if config.diagonal == Some(index) && value > 0.0 {
                value / (1.0 + config.stability_weight)
            } else {
                value
            };
            let thresholded = if biased.abs() >= config.sparse.threshold { biased } else { 0.0 };
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
    use super::*;
    use crate::sr3;

    fn growth_problem() -> RegressionProblem {
        // A scalar self-amplifying relation y ≈ 1.5*x0 + 0.5*x1 where x0 is the
        // linear self-feedback term. A stability bias should shrink the positive
        // self-feedback coefficient toward zero.
        let rows =
            vec![vec![1.0, 0.0], vec![2.0, 1.0], vec![3.0, 0.5], vec![4.0, 2.0], vec![5.0, 1.5]];
        let target = rows.iter().map(|row| 1.5 * row[0] + 0.5 * row[1]).collect::<Vec<_>>();
        RegressionProblem::new(rows, target).unwrap()
    }

    #[test]
    fn biases_positive_self_feedback_toward_stability() {
        let problem = growth_problem();
        let sparse = SparseConfig { threshold: 1e-6, max_iterations: 50, ridge: 1e-8 };
        let baseline = sr3(&problem, &sparse).unwrap();
        let biased = trapping(
            &problem,
            &TrappingConfig { sparse: sparse.clone(), diagonal: Some(0), stability_weight: 1.0 },
        )
        .unwrap();
        // The self-feedback coefficient is pulled below the unbiased SR3 fit while
        // remaining a fit (residual stays finite and bounded).
        assert!(baseline.coefficients[0] > 0.0);
        assert!(biased.coefficients[0] < baseline.coefficients[0]);
        assert!(biased.coefficients[0].is_finite());
    }

    #[test]
    fn reduces_to_sr3_without_a_diagonal() {
        let problem = growth_problem();
        let sparse = SparseConfig { threshold: 1e-6, max_iterations: 50, ridge: 1e-8 };
        let baseline = sr3(&problem, &sparse).unwrap();
        let neutral = trapping(
            &problem,
            &TrappingConfig { sparse: sparse.clone(), diagonal: None, stability_weight: 1.0 },
        )
        .unwrap();
        assert_eq!(neutral.coefficients, baseline.coefficients);
    }

    #[test]
    fn is_deterministic() {
        let problem = growth_problem();
        let config = TrappingConfig {
            sparse: SparseConfig { threshold: 1e-6, max_iterations: 50, ridge: 1e-8 },
            diagonal: Some(0),
            stability_weight: 2.0,
        };
        let first = trapping(&problem, &config).unwrap();
        let second = trapping(&problem, &config).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn rejects_out_of_range_diagonal() {
        let problem = growth_problem();
        let config = TrappingConfig { diagonal: Some(9), ..TrappingConfig::default() };
        assert_eq!(trapping(&problem, &config), Err(SparseError::InvalidConfig));
    }
}
