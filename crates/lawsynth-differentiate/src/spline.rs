use crate::DifferentiationError;

/// Estimates derivatives at sample knots using a natural cubic spline.
///
/// The tridiagonal system uses zero second derivative at both endpoints and
/// accepts non-uniform, strictly increasing sample times.
pub fn cubic_spline_derivative(
    time: &[f64],
    values: &[f64],
) -> Result<Vec<f64>, DifferentiationError> {
    if time.len() != values.len() {
        return Err(DifferentiationError::LengthMismatch);
    }
    if time.len() < 3 {
        return Err(DifferentiationError::TooFewSamples);
    }
    let intervals = time.windows(2).map(|pair| pair[1] - pair[0]).collect::<Vec<_>>();
    if intervals.iter().any(|step| !step.is_finite() || *step <= 0.0)
        || values.iter().any(|value| !value.is_finite())
    {
        return Err(DifferentiationError::SingularFit);
    }
    let count = values.len();
    let mut second = vec![0.0; count];
    if count > 2 {
        let unknowns = count - 2;
        let mut lower = vec![0.0; unknowns];
        let mut diagonal = vec![0.0; unknowns];
        let mut upper = vec![0.0; unknowns];
        let mut right = vec![0.0; unknowns];
        for position in 0..unknowns {
            let index = position + 1;
            let left_step = intervals[index - 1];
            let right_step = intervals[index];
            lower[position] = left_step;
            diagonal[position] = 2.0 * (left_step + right_step);
            upper[position] = right_step;
            right[position] = 6.0
                * ((values[index + 1] - values[index]) / right_step
                    - (values[index] - values[index - 1]) / left_step);
        }
        for position in 1..unknowns {
            let factor = lower[position] / diagonal[position - 1];
            diagonal[position] -= factor * upper[position - 1];
            right[position] -= factor * right[position - 1];
        }
        if diagonal.iter().any(|value| value.abs() <= f64::EPSILON) {
            return Err(DifferentiationError::SingularFit);
        }
        second[count - 2] = right[unknowns - 1] / diagonal[unknowns - 1];
        for position in (0..unknowns - 1).rev() {
            second[position + 1] =
                (right[position] - upper[position] * second[position + 2]) / diagonal[position];
        }
    }
    let mut derivative = vec![0.0; count];
    for index in 0..count - 1 {
        derivative[index] = (values[index + 1] - values[index]) / intervals[index]
            - intervals[index] * (2.0 * second[index] + second[index + 1]) / 6.0;
    }
    let last = count - 1;
    derivative[last] = (values[last] - values[last - 1]) / intervals[last - 1]
        + intervals[last - 1] * (second[last - 1] + 2.0 * second[last]) / 6.0;
    Ok(derivative)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn differentiates_a_cubic_with_natural_boundaries() {
        let time = [0.0, 1.0, 2.0, 3.0];
        let values = time.iter().map(|value| value * value).collect::<Vec<_>>();
        let derivative = cubic_spline_derivative(&time, &values).unwrap();
        for (actual, expected) in derivative.into_iter().zip([0.6, 1.8, 4.2, 5.4]) {
            assert!((actual - expected).abs() < 1e-12);
        }
    }

    #[test]
    fn supports_an_irregular_grid() {
        let time = [0.0, 0.5, 2.0, 3.0];
        let values = time.iter().map(|value| 3.0 * value + 2.0).collect::<Vec<_>>();
        let derivative = cubic_spline_derivative(&time, &values).unwrap();
        assert!(derivative.iter().all(|value| (*value - 3.0).abs() < 1e-12));
    }
}
