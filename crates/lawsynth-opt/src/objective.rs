use crate::OptimizationError;

/// Computes sum of squared residuals for aligned finite vectors.
pub fn residual_sum_squares(predicted: &[f64], observed: &[f64]) -> Result<f64, OptimizationError> {
    if predicted.is_empty() {
        return Err(OptimizationError::EmptyInput);
    }
    if predicted.len() != observed.len() {
        return Err(OptimizationError::LengthMismatch);
    }
    if predicted
        .iter()
        .chain(observed)
        .any(|value| !value.is_finite())
    {
        return Err(OptimizationError::NonFiniteInput);
    }
    Ok(predicted
        .iter()
        .zip(observed)
        .map(|(predicted, observed)| (predicted - observed).powi(2))
        .sum())
}

/// Computes population mean squared error for aligned finite vectors.
pub fn mean_squared_error(predicted: &[f64], observed: &[f64]) -> Result<f64, OptimizationError> {
    Ok(residual_sum_squares(predicted, observed)? / predicted.len() as f64)
}
