use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

use crate::{DerivativeConfig, DerivativeMethod, DifferentiationError};

/// Estimates derivatives at every sample. Interior values use the exact
/// three-point Lagrange derivative, including for irregular time axes.
pub fn differentiate_series(
    time: &[f64],
    values: &[f64],
) -> Result<Vec<f64>, DifferentiationError> {
    if time.len() != values.len() {
        return Err(DifferentiationError::LengthMismatch);
    }
    if time.len() < 2 {
        return Err(DifferentiationError::TooFewSamples);
    }
    let mut result = vec![0.0; values.len()];
    result[0] = (values[1] - values[0]) / (time[1] - time[0]);
    let last = values.len() - 1;
    result[last] = (values[last] - values[last - 1]) / (time[last] - time[last - 1]);
    for index in 1..last {
        let left_step = time[index] - time[index - 1];
        let right_step = time[index + 1] - time[index];
        result[index] = -right_step * values[index - 1] / (left_step * (left_step + right_step))
            + (right_step - left_step) * values[index] / (left_step * right_step)
            + left_step * values[index + 1] / (right_step * (left_step + right_step));
    }
    Ok(result)
}

/// Produces an aligned derivative dataset retaining source column identifiers.
pub fn differentiate_dataset(dataset: &Dataset) -> Result<Dataset, DifferentiationError> {
    differentiate_dataset_with_config(dataset, &DerivativeConfig::default())
}

/// Differentiates every column using the configured deterministic method.
pub fn differentiate_dataset_with_config(
    dataset: &Dataset,
    config: &DerivativeConfig,
) -> Result<Dataset, DifferentiationError> {
    let time = dataset.time().values();
    let columns = dataset
        .columns()
        .values()
        .map(|column| {
            Ok(NumericColumn {
                id: column.id.clone(),
                values: match config.method {
                    DerivativeMethod::FiniteDifference => {
                        differentiate_series(time, &column.values)?
                    }
                    DerivativeMethod::SavitzkyGolay { window } => {
                        crate::savgol_series(time, &column.values, window)?
                    }
                    DerivativeMethod::NaturalCubicSpline => {
                        crate::cubic_spline_derivative(time, &column.values)?
                    }
                    DerivativeMethod::Spectral => crate::spectral_derivative(time, &column.values)?,
                    DerivativeMethod::TotalVariation { lambda, iterations } => {
                        crate::tvreg_series(time, &column.values, lambda, iterations)?
                    }
                },
                unit: column.unit.clone(),
            })
        })
        .collect::<Result<Vec<_>, DifferentiationError>>()?;
    Dataset::new(
        TimeAxis::new(time.to_vec()).expect("source time axis is valid"),
        columns,
    )
    .map_err(|_| DifferentiationError::LengthMismatch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differentiates_a_quadratic_on_an_irregular_grid() {
        let time = [0.0, 0.5, 2.0, 3.0];
        let values = time.iter().map(|value| value * value).collect::<Vec<_>>();
        let derivative = differentiate_series(&time, &values).unwrap();
        assert_eq!(derivative, vec![0.5, 1.0, 4.0, 5.0]);
    }
}
