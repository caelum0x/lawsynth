use crate::{BootstrapResult, IntervalConfig, UncertaintyError};

/// Linearly interpolated empirical quantile. `probability` is in [0, 1].
pub fn percentile(values: &[f64], probability: f64) -> Result<f64, UncertaintyError> {
    if values.is_empty() {
        return Err(UncertaintyError::EmptyInput);
    }
    if values.iter().any(|value| !value.is_finite()) {
        return Err(UncertaintyError::NonFiniteValue);
    }
    if !probability.is_finite() || !(0.0..=1.0).contains(&probability) {
        return Err(UncertaintyError::InvalidConfidence(probability));
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let rank = probability * (sorted.len() - 1) as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    Ok(sorted[lower] + (sorted[upper] - sorted[lower]) * (rank - lower as f64))
}

/// Central percentile interval from bootstrap estimates.
pub fn confidence_interval(
    result: &BootstrapResult,
    config: IntervalConfig,
) -> Result<(f64, f64), UncertaintyError> {
    config.validate()?;
    if result.estimates.len() < 2 {
        return Err(UncertaintyError::InsufficientResamples);
    }
    let tail = (1.0 - config.confidence) / 2.0;
    Ok((
        percentile(&result.estimates, tail)?,
        percentile(&result.estimates, 1.0 - tail)?,
    ))
}
