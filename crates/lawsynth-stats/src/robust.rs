use crate::{StatsError, median, quantile};

/// Median absolute deviation around the sample median.
pub fn median_absolute_deviation(values: &[f64]) -> Result<f64, StatsError> {
    let center = median(values)?;
    median(&values.iter().map(|value| (value - center).abs()).collect::<Vec<_>>())
}

/// Replaces tail values with the inclusive quantile bounds.
pub fn winsorize(values: &[f64], tail_fraction: f64) -> Result<Vec<f64>, StatsError> {
    if !tail_fraction.is_finite() || !(0.0..0.5).contains(&tail_fraction) {
        return Err(StatsError::InvalidProbability);
    }
    let lower = quantile(values, tail_fraction)?;
    let upper = quantile(values, 1.0 - tail_fraction)?;
    Ok(values.iter().map(|value| value.clamp(lower, upper)).collect())
}
