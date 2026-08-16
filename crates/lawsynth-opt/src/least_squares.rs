use crate::OptimizationError;

/// The least-squares affine calibration `target ~= scale * prediction + offset`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AffineFit {
    pub scale: f64,
    pub offset: f64,
    pub mean_squared_error: f64,
}

/// Fits two scalar constants analytically without iteration or random state.
pub fn fit_affine(predictions: &[f64], targets: &[f64]) -> Result<AffineFit, OptimizationError> {
    if predictions.is_empty() {
        return Err(OptimizationError::EmptyInput);
    }
    if predictions.len() != targets.len() {
        return Err(OptimizationError::LengthMismatch);
    }
    if predictions
        .iter()
        .chain(targets)
        .any(|value| !value.is_finite())
    {
        return Err(OptimizationError::NonFiniteInput);
    }
    let count = predictions.len() as f64;
    let prediction_mean = predictions.iter().sum::<f64>() / count;
    let target_mean = targets.iter().sum::<f64>() / count;
    let (covariance, variance) = predictions.iter().zip(targets).fold(
        (0.0, 0.0),
        |(covariance, variance), (prediction, target)| {
            let centered_prediction = prediction - prediction_mean;
            (
                covariance + centered_prediction * (target - target_mean),
                variance + centered_prediction * centered_prediction,
            )
        },
    );
    if variance <= f64::EPSILON {
        return Err(OptimizationError::DegeneratePredictor);
    }
    let scale = covariance / variance;
    let offset = target_mean - scale * prediction_mean;
    let mean_squared_error = predictions
        .iter()
        .zip(targets)
        .map(|(prediction, target)| {
            let residual = scale * prediction + offset - target;
            residual * residual
        })
        .sum::<f64>()
        / count;
    Ok(AffineFit {
        scale,
        offset,
        mean_squared_error,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibrates_a_symbolic_candidate() {
        let fit = fit_affine(&[1.0, 2.0, 3.0], &[5.0, 7.0, 9.0]).unwrap();
        assert_eq!(fit.scale, 2.0);
        assert_eq!(fit.offset, 3.0);
        assert_eq!(fit.mean_squared_error, 0.0);
    }
}
