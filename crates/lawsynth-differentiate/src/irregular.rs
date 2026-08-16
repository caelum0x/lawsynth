use crate::{DifferentiationError, differentiate_series};

/// Explicit irregular-grid derivative entry point.
///
/// Unlike the spectral method this validates strictly increasing, finite sample
/// times before invoking the three-point Lagrange estimator.
pub fn irregular_three_point_derivative(
    time: &[f64],
    values: &[f64],
) -> Result<Vec<f64>, DifferentiationError> {
    if time.len() != values.len() {
        return Err(DifferentiationError::LengthMismatch);
    }
    if time.len() < 2 {
        return Err(DifferentiationError::TooFewSamples);
    }
    if time
        .windows(2)
        .any(|pair| !pair[0].is_finite() || !pair[1].is_finite() || pair[1] <= pair[0])
        || values.iter().any(|value| !value.is_finite())
    {
        return Err(DifferentiationError::SingularFit);
    }
    differentiate_series(time, values)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_nonuniform_monotone_samples() {
        assert_eq!(
            irregular_three_point_derivative(&[0.0, 0.5, 2.0], &[1.0, 2.0, 5.0]).unwrap(),
            vec![2.0; 3]
        );
    }
}
