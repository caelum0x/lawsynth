use crate::{ScoreError, fit::validate_pair};

/// Distributional summary of signed residuals.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResidualSummary {
    pub mean: f64,
    pub standard_deviation: f64,
    pub maximum_absolute: f64,
}

/// Returns `observed - predicted` in the original observation order.
pub fn residuals(observed: &[f64], predicted: &[f64]) -> Result<Vec<f64>, ScoreError> {
    validate_pair(observed, predicted)?;
    Ok(observed.iter().zip(predicted).map(|(actual, estimate)| actual - estimate).collect())
}

impl ResidualSummary {
    pub fn from_residuals(values: &[f64]) -> Result<Self, ScoreError> {
        if values.is_empty() {
            return Err(ScoreError::EmptyObservations);
        }
        if values.iter().any(|value| !value.is_finite()) {
            return Err(ScoreError::NonFiniteValue);
        }
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance =
            values.iter().map(|value| (value - mean).powi(2)).sum::<f64>() / values.len() as f64;
        Ok(Self {
            mean,
            standard_deviation: variance.sqrt(),
            maximum_absolute: values.iter().map(|value| value.abs()).fold(0.0, f64::max),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retains_signed_residual_order_and_summarizes_spread() {
        let values = residuals(&[2.0, 1.0], &[1.0, 3.0]).unwrap();
        assert_eq!(values, vec![1.0, -2.0]);
        assert_eq!(ResidualSummary::from_residuals(&values).unwrap().maximum_absolute, 2.0);
    }
}
