use crate::bootstrap::next_u64;
use crate::{CovarianceMatrix, PropagationConfig, Samples, UncertaintyError};

/// First-order propagation: sqrt(gradientᵀ covariance gradient).
pub fn linear_propagate(
    gradient: &[f64],
    covariance: &CovarianceMatrix,
) -> Result<f64, UncertaintyError> {
    let variance = covariance.quadratic_form(gradient)?;
    if variance < -1e-12 {
        return Err(UncertaintyError::NonPositiveVariance);
    }
    Ok(variance.max(0.0).sqrt())
}

/// Empirically propagate independent input sample distributions through `model`.
/// Each input column is sampled with replacement; no correlation is implied.
pub fn monte_carlo_propagate<F>(
    inputs: &[Samples],
    config: PropagationConfig,
    model: F,
) -> Result<Samples, UncertaintyError>
where
    F: Fn(&[f64]) -> f64,
{
    config.validate()?;
    if inputs.is_empty() {
        return Err(UncertaintyError::EmptyInput);
    }
    let mut state = config.seed;
    let mut point = vec![0.0; inputs.len()];
    let mut output = Vec::with_capacity(config.draws);
    for _ in 0..config.draws {
        for (target, input) in point.iter_mut().zip(inputs) {
            *target = input.as_slice()[(next_u64(&mut state) % input.len() as u64) as usize];
        }
        let value = model(&point);
        if !value.is_finite() {
            return Err(UncertaintyError::NonFiniteValue);
        }
        output.push(value);
    }
    Samples::new(output)
}
