use crate::StatsError;

/// Computes a linearly interpolated quantile after sorting finite values.
pub fn quantile(values: &[f64], probability: f64) -> Result<f64, StatsError> {
    if values.is_empty() {
        return Err(StatsError::EmptyInput);
    }
    if !probability.is_finite() || !(0.0..=1.0).contains(&probability) {
        return Err(StatsError::InvalidProbability);
    }
    if values.iter().any(|value| !value.is_finite()) {
        return Err(StatsError::NonFiniteValue);
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    Ok(quantile_sorted(&sorted, probability))
}

/// Computes a linear quantile from already sorted finite values.
pub fn quantile_sorted(values: &[f64], probability: f64) -> f64 {
    debug_assert!(!values.is_empty());
    let position = probability * (values.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    values[lower] + (values[upper] - values[lower]) * (position - lower as f64)
}

pub fn median(values: &[f64]) -> Result<f64, StatsError> {
    quantile(values, 0.5)
}
