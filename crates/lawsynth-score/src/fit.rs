use crate::ScoreError;

/// Standard regression fit measures calculated from aligned observations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FitStatistics {
    pub mean_absolute_error: f64,
    pub mean_squared_error: f64,
    pub root_mean_squared_error: f64,
    pub r_squared: f64,
    pub residual_sum_squares: f64,
}

/// Computes finite, population-normalized fit statistics.
pub fn fit_statistics(observed: &[f64], predicted: &[f64]) -> Result<FitStatistics, ScoreError> {
    validate_pair(observed, predicted)?;
    let count = observed.len() as f64;
    let mean = observed.iter().sum::<f64>() / count;
    let mut absolute_sum = 0.0;
    let mut residual_sum_squares = 0.0;
    let mut total_sum_squares = 0.0;
    for (actual, estimate) in observed.iter().zip(predicted) {
        let residual = actual - estimate;
        absolute_sum += residual.abs();
        residual_sum_squares += residual * residual;
        let centered = actual - mean;
        total_sum_squares += centered * centered;
    }
    let mean_squared_error = residual_sum_squares / count;
    Ok(FitStatistics {
        mean_absolute_error: absolute_sum / count,
        mean_squared_error,
        root_mean_squared_error: mean_squared_error.sqrt(),
        r_squared: if total_sum_squares > 0.0 {
            1.0 - residual_sum_squares / total_sum_squares
        } else if residual_sum_squares == 0.0 {
            1.0
        } else {
            0.0
        },
        residual_sum_squares,
    })
}

pub(crate) fn validate_pair(observed: &[f64], predicted: &[f64]) -> Result<(), ScoreError> {
    if observed.is_empty() {
        return Err(ScoreError::EmptyObservations);
    }
    if observed.len() != predicted.len() {
        return Err(ScoreError::LengthMismatch);
    }
    if observed
        .iter()
        .chain(predicted)
        .any(|value| !value.is_finite())
    {
        return Err(ScoreError::NonFiniteValue);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_regression_fit_without_special_case_division() {
        let stats = fit_statistics(&[1.0, 2.0, 3.0], &[1.0, 1.0, 4.0]).unwrap();
        assert!((stats.mean_squared_error - 2.0 / 3.0).abs() < 1e-12);
        assert_eq!(stats.r_squared, 0.0);
    }
}
