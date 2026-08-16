use crate::{OptimizationError, ParameterBounds};

/// Deterministic Nelder-Mead simplex controls for derivative-free objectives.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NelderMeadConfig {
    pub initial_step: f64,
    pub tolerance: f64,
    pub max_iterations: usize,
}
impl Default for NelderMeadConfig {
    fn default() -> Self {
        Self {
            initial_step: 1.0,
            tolerance: 1e-8,
            max_iterations: 1_000,
        }
    }
}

/// Minimizes a bounded objective with reflection, expansion, contraction, and shrink steps.
pub fn nelder_mead_minimize<F>(
    initial: &[f64],
    bounds: ParameterBounds,
    config: NelderMeadConfig,
    objective: F,
) -> Result<Vec<f64>, OptimizationError>
where
    F: Fn(&[f64]) -> f64,
{
    if initial.is_empty() {
        return Err(OptimizationError::EmptyInput);
    }
    if initial.iter().any(|value| !value.is_finite())
        || !config.initial_step.is_finite()
        || config.initial_step <= 0.0
        || !config.tolerance.is_finite()
        || config.tolerance <= 0.0
        || config.max_iterations == 0
    {
        return Err(OptimizationError::InvalidConfig);
    }
    let dimension = initial.len();
    let mut simplex = Vec::with_capacity(dimension + 1);
    simplex.push(
        initial
            .iter()
            .map(|value| bounds.clamp(*value))
            .collect::<Vec<_>>(),
    );
    for coordinate in 0..dimension {
        let mut point = simplex[0].clone();
        point[coordinate] = bounds.clamp(point[coordinate] + config.initial_step);
        simplex.push(point);
    }
    let mut scores = simplex
        .iter()
        .map(|point| score(&objective, point))
        .collect::<Result<Vec<_>, _>>()?;
    for _ in 0..config.max_iterations {
        let mut order = (0..simplex.len()).collect::<Vec<_>>();
        order.sort_by(|left, right| {
            scores[*left]
                .total_cmp(&scores[*right])
                .then_with(|| left.cmp(right))
        });
        simplex = order.iter().map(|index| simplex[*index].clone()).collect();
        scores = order.iter().map(|index| scores[*index]).collect();
        if scores[dimension] - scores[0] <= config.tolerance {
            return Ok(simplex[0].clone());
        }
        let centroid = (0..dimension)
            .map(|coordinate| {
                simplex[..dimension]
                    .iter()
                    .map(|point| point[coordinate])
                    .sum::<f64>()
                    / dimension as f64
            })
            .collect::<Vec<_>>();
        let trial = |factor: f64| {
            centroid
                .iter()
                .zip(&simplex[dimension])
                .map(|(center, worst)| bounds.clamp(center + factor * (center - worst)))
                .collect::<Vec<_>>()
        };
        let reflected = trial(1.0);
        let reflected_score = score(&objective, &reflected)?;
        if reflected_score < scores[0] {
            let expanded = trial(2.0);
            let expanded_score = score(&objective, &expanded)?;
            if expanded_score < reflected_score {
                simplex[dimension] = expanded;
                scores[dimension] = expanded_score;
            } else {
                simplex[dimension] = reflected;
                scores[dimension] = reflected_score;
            }
        } else if reflected_score < scores[dimension - 1] {
            simplex[dimension] = reflected;
            scores[dimension] = reflected_score;
        } else {
            let contracted = trial(0.5);
            let contracted_score = score(&objective, &contracted)?;
            if contracted_score < scores[dimension] {
                simplex[dimension] = contracted;
                scores[dimension] = contracted_score;
            } else {
                let best_point = simplex[0].clone();
                for index in 1..=dimension {
                    for (value, best) in simplex[index].iter_mut().zip(&best_point) {
                        *value = bounds.clamp(0.5 * (*value + *best));
                    }
                    scores[index] = score(&objective, &simplex[index])?;
                }
            }
        }
    }
    let best = scores
        .iter()
        .enumerate()
        .min_by(|left, right| left.1.total_cmp(right.1).then_with(|| left.0.cmp(&right.0)))
        .expect("nonempty simplex")
        .0;
    Ok(simplex[best].clone())
}
fn score<F>(objective: &F, point: &[f64]) -> Result<f64, OptimizationError>
where
    F: Fn(&[f64]) -> f64,
{
    let value = objective(point);
    value
        .is_finite()
        .then_some(value)
        .ok_or(OptimizationError::NonFiniteObjective)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn minimizes_smooth_bounded_objective() {
        let result = nelder_mead_minimize(
            &[0.0, 0.0],
            ParameterBounds::new(-5.0, 5.0).unwrap(),
            NelderMeadConfig::default(),
            |p| (p[0] - 1.5).powi(2) + (p[1] + 2.0).powi(2),
        )
        .unwrap();
        assert!((result[0] - 1.5).abs() < 1e-5 && (result[1] + 2.0).abs() < 1e-5);
    }
}
