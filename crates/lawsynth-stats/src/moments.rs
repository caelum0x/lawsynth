use crate::StatsError;

/// Stable descriptive moments of a finite scalar sample.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MomentSummary {
    pub count: usize,
    pub minimum: f64,
    pub maximum: f64,
    pub mean: f64,
    pub population_variance: f64,
    pub sample_variance: f64,
}

/// Computes moments with Welford's one-pass update.
pub fn moments(values: &[f64]) -> Result<MomentSummary, StatsError> {
    let Some((&first, rest)) = values.split_first() else {
        return Err(StatsError::EmptyInput);
    };
    if !first.is_finite() {
        return Err(StatsError::NonFiniteValue);
    }
    let mut count = 1_usize;
    let mut minimum = first;
    let mut maximum = first;
    let mut mean = first;
    let mut squared_deviation = 0.0;
    for value in rest {
        if !value.is_finite() {
            return Err(StatsError::NonFiniteValue);
        }
        count += 1;
        minimum = minimum.min(*value);
        maximum = maximum.max(*value);
        let delta = *value - mean;
        mean += delta / count as f64;
        squared_deviation += delta * (*value - mean);
    }
    Ok(MomentSummary {
        count,
        minimum,
        maximum,
        mean,
        population_variance: squared_deviation / count as f64,
        sample_variance: if count > 1 {
            squared_deviation / (count - 1) as f64
        } else {
            0.0
        },
    })
}
