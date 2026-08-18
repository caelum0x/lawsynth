use crate::DifferentiationError;

/// Evaluates a weak-form derivative integral using integration by parts.
///
/// Returns `x(t_end)φ(t_end) - x(t_start)φ(t_start) - ∫ x(t)φ'(t)dt`, the
/// weak observation of `∫ x'(t)φ(t)dt`; all integrals use the trapezoid rule
/// over the supplied (possibly irregular) timestamps.
pub fn weak_derivative_integral(
    time: &[f64],
    values: &[f64],
    test: &[f64],
    test_derivative: &[f64],
) -> Result<f64, DifferentiationError> {
    if time.len() != values.len()
        || values.len() != test.len()
        || test.len() != test_derivative.len()
    {
        return Err(DifferentiationError::LengthMismatch);
    }
    if time.len() < 2 {
        return Err(DifferentiationError::TooFewSamples);
    }
    if time
        .windows(2)
        .any(|pair| !pair[0].is_finite() || !pair[1].is_finite() || pair[1] <= pair[0])
        || values.iter().chain(test).chain(test_derivative).any(|value| !value.is_finite())
    {
        return Err(DifferentiationError::SingularFit);
    }
    let integral = (0..time.len() - 1)
        .map(|index| {
            let dt = time[index + 1] - time[index];
            0.5 * dt
                * (values[index] * test_derivative[index]
                    + values[index + 1] * test_derivative[index + 1])
        })
        .sum::<f64>();
    Ok(values[values.len() - 1] * test[test.len() - 1] - values[0] * test[0] - integral)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recovers_linear_derivative_against_constant_test_function() {
        assert_eq!(
            weak_derivative_integral(&[0.0, 1.0, 2.0], &[1.0, 3.0, 5.0], &[1.0; 3], &[0.0; 3])
                .unwrap(),
            4.0
        );
    }
}
