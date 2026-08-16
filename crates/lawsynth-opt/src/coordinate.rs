use crate::{CoordinateConfig, OptimizationError, ParameterBounds, TerminationReason};

/// Result of deterministic bounded coordinate minimization.
#[derive(Clone, Debug, PartialEq)]
pub struct CoordinateResult {
    pub parameters: Vec<f64>,
    pub objective: f64,
    pub iterations: usize,
    pub termination: TerminationReason,
}

/// Minimizes a finite scalar objective with plus/minus coordinate proposals.
///
/// Coordinates are visited left-to-right, ties prefer the negative proposal,
/// and an unproductive full sweep halves the common step size.
pub fn coordinate_minimize<F>(
    initial: &[f64],
    bounds: ParameterBounds,
    config: CoordinateConfig,
    objective: F,
) -> Result<CoordinateResult, OptimizationError>
where
    F: Fn(&[f64]) -> f64,
{
    if initial.is_empty() {
        return Err(OptimizationError::EmptyInput);
    }
    if initial.iter().any(|value| !value.is_finite()) {
        return Err(OptimizationError::NonFiniteInput);
    }
    if !config.initial_step.is_finite()
        || !config.minimum_step.is_finite()
        || config.initial_step <= 0.0
        || config.minimum_step <= 0.0
        || config.minimum_step > config.initial_step
        || config.max_iterations == 0
    {
        return Err(OptimizationError::InvalidConfig);
    }
    let mut parameters = initial
        .iter()
        .map(|value| bounds.clamp(*value))
        .collect::<Vec<_>>();
    let mut best = checked_objective(&objective, &parameters)?;
    let mut step = config.initial_step;
    for iteration in 0..config.max_iterations {
        let mut improved = false;
        for coordinate in 0..parameters.len() {
            let original = parameters[coordinate];
            let negative = bounds.clamp(original - step);
            let positive = bounds.clamp(original + step);
            let negative_score = score_proposal(
                &objective,
                &mut parameters,
                coordinate,
                original,
                negative,
                best,
            )?;
            let positive_score = score_proposal(
                &objective,
                &mut parameters,
                coordinate,
                original,
                positive,
                best,
            )?;
            if negative_score <= positive_score && negative_score < best {
                parameters[coordinate] = negative;
                best = negative_score;
                improved = true;
            } else if positive_score < best {
                parameters[coordinate] = positive;
                best = positive_score;
                improved = true;
            }
        }
        if !improved {
            step *= 0.5;
            if step < config.minimum_step {
                return Ok(CoordinateResult {
                    parameters,
                    objective: best,
                    iterations: iteration + 1,
                    termination: TerminationReason::MinimumStep,
                });
            }
        }
    }
    Ok(CoordinateResult {
        parameters,
        objective: best,
        iterations: config.max_iterations,
        termination: TerminationReason::IterationLimit,
    })
}

fn score_proposal<F>(
    objective: &F,
    parameters: &mut [f64],
    coordinate: usize,
    original: f64,
    proposal: f64,
    current: f64,
) -> Result<f64, OptimizationError>
where
    F: Fn(&[f64]) -> f64,
{
    if proposal == original {
        return Ok(current);
    }
    parameters[coordinate] = proposal;
    let score = checked_objective(objective, parameters)?;
    parameters[coordinate] = original;
    Ok(score)
}

fn checked_objective<F>(objective: &F, parameters: &[f64]) -> Result<f64, OptimizationError>
where
    F: Fn(&[f64]) -> f64,
{
    let value = objective(parameters);
    if value.is_finite() {
        Ok(value)
    } else {
        Err(OptimizationError::NonFiniteObjective)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_quadratic_constants_with_bounds() {
        let result = coordinate_minimize(
            &[0.0, 0.0],
            ParameterBounds::new(-10.0, 10.0).unwrap(),
            CoordinateConfig {
                initial_step: 2.0,
                minimum_step: 1e-6,
                max_iterations: 200,
            },
            |parameters| (parameters[0] - 1.5).powi(2) + (parameters[1] + 2.25).powi(2),
        )
        .unwrap();
        assert!((result.parameters[0] - 1.5).abs() < 2e-6);
        assert!((result.parameters[1] + 2.25).abs() < 2e-6);
        assert!(result.objective < 1e-10);
    }
}
