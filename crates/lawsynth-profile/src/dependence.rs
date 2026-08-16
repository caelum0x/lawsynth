use crate::ProfileError;

/// Population Pearson correlation for two aligned numeric observations.
pub fn pearson_correlation(left: &[f64], right: &[f64]) -> Result<f64, ProfileError> {
    if left.len() != right.len() {
        return Err(ProfileError::LengthMismatch);
    }
    if left.len() < 2 {
        return Err(ProfileError::TooFewValues);
    }
    let count = left.len() as f64;
    let left_mean = left.iter().sum::<f64>() / count;
    let right_mean = right.iter().sum::<f64>() / count;
    let (covariance, left_variance, right_variance) = left.iter().zip(right).fold(
        (0.0, 0.0, 0.0),
        |(covariance, left_variance, right_variance), (left, right)| {
            let left_delta = left - left_mean;
            let right_delta = right - right_mean;
            (
                covariance + left_delta * right_delta,
                left_variance + left_delta * left_delta,
                right_variance + right_delta * right_delta,
            )
        },
    );
    if left_variance <= f64::EPSILON || right_variance <= f64::EPSILON {
        return Err(ProfileError::ConstantValues);
    }
    Ok(covariance / (left_variance * right_variance).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_perfect_anti_correlation() {
        assert_eq!(
            pearson_correlation(&[1.0, 2.0, 3.0], &[3.0, 2.0, 1.0]).unwrap(),
            -1.0
        );
    }
}
