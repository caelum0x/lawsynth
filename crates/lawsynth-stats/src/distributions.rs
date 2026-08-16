use crate::StatsError;

/// Probability density of a normal distribution at a finite scalar.
pub fn normal_pdf(value: f64, mean: f64, standard_deviation: f64) -> Result<f64, StatsError> {
    validate_normal_arguments(value, mean, standard_deviation)?;
    let normalized = (value - mean) / standard_deviation;
    Ok((-0.5 * normalized * normalized).exp()
        / (standard_deviation * (2.0 * std::f64::consts::PI).sqrt()))
}

/// Cumulative probability of a normal distribution using a maximum-error
/// rational approximation to `erf`.
pub fn normal_cdf(value: f64, mean: f64, standard_deviation: f64) -> Result<f64, StatsError> {
    validate_normal_arguments(value, mean, standard_deviation)?;
    let x = (value - mean) / (standard_deviation * 2.0_f64.sqrt());
    Ok(0.5 * (1.0 + erf_approximation(x)))
}

fn validate_normal_arguments(
    value: f64,
    mean: f64,
    standard_deviation: f64,
) -> Result<(), StatsError> {
    if !value.is_finite() || !mean.is_finite() {
        return Err(StatsError::NonFiniteValue);
    }
    if !standard_deviation.is_finite() || standard_deviation <= 0.0 {
        return Err(StatsError::InvalidStandardDeviation);
    }
    Ok(())
}

fn erf_approximation(value: f64) -> f64 {
    let sign = value.signum();
    let x = value.abs();
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let polynomial =
        (((((1.061_405_429 * t - 1.453_152_027) * t) + 1.421_413_741) * t - 0.284_496_736) * t
            + 0.254_829_592)
            * t;
    sign * (1.0 - polynomial * (-x * x).exp())
}
