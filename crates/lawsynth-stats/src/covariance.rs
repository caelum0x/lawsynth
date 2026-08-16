use crate::{StatsError, moments};

/// Population covariance of two aligned finite samples.
pub fn covariance(left: &[f64], right: &[f64]) -> Result<f64, StatsError> {
    if left.len() != right.len() {
        return Err(StatsError::LengthMismatch);
    }
    if left.len() < 2 {
        return Err(StatsError::TooFewValues);
    }
    let left_moments = moments(left)?;
    let right_moments = moments(right)?;
    Ok(left
        .iter()
        .zip(right)
        .map(|(a, b)| (a - left_moments.mean) * (b - right_moments.mean))
        .sum::<f64>()
        / left.len() as f64)
}

/// Population Pearson correlation of two aligned finite samples.
pub fn pearson_correlation(left: &[f64], right: &[f64]) -> Result<f64, StatsError> {
    if left.len() != right.len() {
        return Err(StatsError::LengthMismatch);
    }
    let left_moments = moments(left)?;
    let right_moments = moments(right)?;
    if left_moments.count < 2 {
        return Err(StatsError::TooFewValues);
    }
    if left_moments.population_variance <= f64::EPSILON
        || right_moments.population_variance <= f64::EPSILON
    {
        return Err(StatsError::ConstantValues);
    }
    Ok(covariance(left, right)?
        / (left_moments.population_variance * right_moments.population_variance).sqrt())
}
