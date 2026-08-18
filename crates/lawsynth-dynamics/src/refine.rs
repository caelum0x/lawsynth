use lawsynth_data::TimeAxis;

use crate::DynamicsError;

/// Differentiates a scalar series with second-order centered interior stencils.
/// Boundary values use one-sided secants, preserving the observed sample count.
pub fn central_derivative(time: &TimeAxis, values: &[f64]) -> Result<Vec<f64>, DynamicsError> {
    if values.len() != time.len() || values.len() < 2 {
        return Err(DynamicsError::TooFewSamples);
    }
    if values.iter().any(|value| !value.is_finite()) {
        return Err(DynamicsError::NonFiniteValue);
    }
    let times = time.values();
    let mut result = Vec::with_capacity(values.len());
    result.push((values[1] - values[0]) / (times[1] - times[0]));
    for index in 1..values.len() - 1 {
        result
            .push((values[index + 1] - values[index - 1]) / (times[index + 1] - times[index - 1]));
    }
    let last = values.len() - 1;
    result.push((values[last] - values[last - 1]) / (times[last] - times[last - 1]));
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lawsynth_data::TimeAxis;
    #[test]
    fn differentiates_linear_data_on_an_irregular_axis() {
        let time = TimeAxis::new(vec![0.0, 0.5, 2.0]).unwrap();
        assert_eq!(central_derivative(&time, &[1.0, 2.0, 5.0]).unwrap(), vec![2.0; 3]);
    }
}
