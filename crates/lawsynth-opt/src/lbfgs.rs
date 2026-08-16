use crate::{OptimizationError, ParameterBounds};

/// Limited-memory BFGS settings for smooth bounded objectives with analytic gradients.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LbfgsConfig {
    pub memory: usize,
    pub tolerance: f64,
    pub max_iterations: usize,
}
impl Default for LbfgsConfig {
    fn default() -> Self {
        Self {
            memory: 8,
            tolerance: 1e-8,
            max_iterations: 500,
        }
    }
}

/// Minimizes a smooth function using L-BFGS two-loop recursion and Armijo backtracking.
pub fn lbfgs_minimize<F>(
    initial: &[f64],
    bounds: ParameterBounds,
    config: LbfgsConfig,
    function: F,
) -> Result<Vec<f64>, OptimizationError>
where
    F: Fn(&[f64]) -> (f64, Vec<f64>),
{
    if initial.is_empty() {
        return Err(OptimizationError::EmptyInput);
    }
    if config.memory == 0
        || config.max_iterations == 0
        || !config.tolerance.is_finite()
        || config.tolerance <= 0.0
        || initial.iter().any(|value| !value.is_finite())
    {
        return Err(OptimizationError::InvalidConfig);
    }
    let mut point = initial
        .iter()
        .map(|value| bounds.clamp(*value))
        .collect::<Vec<_>>();
    let (mut value, mut gradient) = checked(&function, &point)?;
    let mut history: Vec<(Vec<f64>, Vec<f64>, f64)> = Vec::new();
    for _ in 0..config.max_iterations {
        if norm(&gradient) <= config.tolerance {
            return Ok(point);
        }
        let direction = direction(&gradient, &history);
        let slope = dot(&gradient, &direction);
        if slope >= 0.0 {
            return Ok(point);
        }
        let mut step = 1.0;
        let (next, next_value, next_gradient) = loop {
            let candidate = point
                .iter()
                .zip(&direction)
                .map(|(value, direction)| bounds.clamp(value + step * direction))
                .collect::<Vec<_>>();
            let (candidate_value, candidate_gradient) = checked(&function, &candidate)?;
            if candidate_value <= value + 1e-4 * step * slope || step < 1e-12 {
                break (candidate, candidate_value, candidate_gradient);
            }
            step *= 0.5;
        };
        let s = next
            .iter()
            .zip(&point)
            .map(|(next, prior)| next - prior)
            .collect::<Vec<_>>();
        let y = next_gradient
            .iter()
            .zip(&gradient)
            .map(|(next, prior)| next - prior)
            .collect::<Vec<_>>();
        let curvature = dot(&s, &y);
        if curvature > 1e-14 {
            history.push((s, y, 1.0 / curvature));
            if history.len() > config.memory {
                history.remove(0);
            }
        }
        point = next;
        value = next_value;
        gradient = next_gradient;
    }
    Ok(point)
}
fn checked<F>(function: &F, point: &[f64]) -> Result<(f64, Vec<f64>), OptimizationError>
where
    F: Fn(&[f64]) -> (f64, Vec<f64>),
{
    let (value, gradient) = function(point);
    if !value.is_finite()
        || gradient.len() != point.len()
        || gradient.iter().any(|value| !value.is_finite())
    {
        Err(OptimizationError::NonFiniteObjective)
    } else {
        Ok((value, gradient))
    }
}
fn direction(gradient: &[f64], history: &[(Vec<f64>, Vec<f64>, f64)]) -> Vec<f64> {
    let mut q = gradient.to_vec();
    let mut alpha = Vec::new();
    for (s, y, rho) in history.iter().rev() {
        let value = rho * dot(s, &q);
        alpha.push(value);
        for (q, y) in q.iter_mut().zip(y) {
            *q -= value * y;
        }
    }
    let scale = history
        .last()
        .map(|(s, y, _)| dot(s, y) / dot(y, y))
        .filter(|value| value.is_finite() && *value > 0.0)
        .unwrap_or(1.0);
    for value in &mut q {
        *value *= scale;
    }
    for ((s, y, rho), alpha) in history.iter().zip(alpha.into_iter().rev()) {
        let beta = rho * dot(y, &q);
        for (q, s) in q.iter_mut().zip(s) {
            *q += s * (alpha - beta);
        }
    }
    q.into_iter().map(|value| -value).collect()
}
fn dot(left: &[f64], right: &[f64]) -> f64 {
    left.iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum()
}
fn norm(values: &[f64]) -> f64 {
    dot(values, values).sqrt()
}
