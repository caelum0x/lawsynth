use crate::{DifferentiationError, differentiate_series};

/// Denoises a series by solving the one-dimensional Rudin-Osher-Fatemi total
/// variation problem with a deterministic ADMM iteration.
///
/// The objective is `0.5 * ||x - values||² + lambda * ||D x||₁`, where `D`
/// is the adjacent-sample difference operator. The time axis is validated for
/// alignment because it is needed by [`tvreg_series`], but denoising itself is
/// indexed by adjacent observations rather than their spacing.
pub fn tvreg_smoothed_series(
    values: &[f64],
    lambda: f64,
    iterations: usize,
) -> Result<Vec<f64>, DifferentiationError> {
    if values.len() < 2 {
        return Err(DifferentiationError::TooFewSamples);
    }
    if !lambda.is_finite() || lambda <= 0.0 || iterations == 0 {
        return Err(DifferentiationError::InvalidTotalVariationConfig);
    }
    if values.iter().any(|value| !value.is_finite()) {
        return Err(DifferentiationError::SingularFit);
    }

    // rho=1 gives a well-conditioned tridiagonal x update while keeping the
    // shrinkage threshold directly interpretable as lambda.
    const RHO: f64 = 1.0;
    const TOLERANCE: f64 = 1e-9;
    let count = values.len();
    let differences = count - 1;
    let mut estimate = values.to_vec();
    let mut auxiliary = vec![0.0; differences];
    let mut dual = vec![0.0; differences];

    for _ in 0..iterations {
        let previous_auxiliary = auxiliary.clone();
        let mut right_hand_side = values.to_vec();
        let adjusted = auxiliary.iter().zip(&dual).map(|(z, u)| z - u).collect::<Vec<_>>();
        right_hand_side[0] -= RHO * adjusted[0];
        for index in 1..count - 1 {
            right_hand_side[index] += RHO * (adjusted[index - 1] - adjusted[index]);
        }
        right_hand_side[count - 1] += RHO * adjusted[differences - 1];
        estimate = solve_identity_plus_difference_laplacian(&right_hand_side, RHO)?;

        let mut primal_residual = 0.0_f64;
        let mut dual_residual = 0.0_f64;
        for index in 0..differences {
            let difference = estimate[index + 1] - estimate[index];
            auxiliary[index] = soft_threshold(difference + dual[index], lambda / RHO);
            let residual = difference - auxiliary[index];
            dual[index] += residual;
            primal_residual = primal_residual.max(residual.abs());
            dual_residual = dual_residual.max((auxiliary[index] - previous_auxiliary[index]).abs());
        }
        if primal_residual <= TOLERANCE && dual_residual <= TOLERANCE {
            break;
        }
    }
    Ok(estimate)
}

/// Estimates a derivative after total-variation regularization of a signal.
pub fn tvreg_series(
    time: &[f64],
    values: &[f64],
    lambda: f64,
    iterations: usize,
) -> Result<Vec<f64>, DifferentiationError> {
    if time.len() != values.len() {
        return Err(DifferentiationError::LengthMismatch);
    }
    let smoothed = tvreg_smoothed_series(values, lambda, iterations)?;
    differentiate_series(time, &smoothed)
}

fn soft_threshold(value: f64, threshold: f64) -> f64 {
    if value > threshold {
        value - threshold
    } else if value < -threshold {
        value + threshold
    } else {
        0.0
    }
}

fn solve_identity_plus_difference_laplacian(
    right_hand_side: &[f64],
    rho: f64,
) -> Result<Vec<f64>, DifferentiationError> {
    let count = right_hand_side.len();
    let mut diagonal = (0..count)
        .map(|index| if index == 0 || index + 1 == count { 1.0 + rho } else { 1.0 + 2.0 * rho })
        .collect::<Vec<_>>();
    let mut right = right_hand_side.to_vec();
    for index in 1..count {
        let factor = -rho / diagonal[index - 1];
        diagonal[index] += rho * factor;
        right[index] -= factor * right[index - 1];
    }
    if diagonal.iter().any(|value| !value.is_finite() || value.abs() <= f64::EPSILON) {
        return Err(DifferentiationError::SingularFit);
    }
    let mut solution = vec![0.0; count];
    solution[count - 1] = right[count - 1] / diagonal[count - 1];
    for index in (0..count - 1).rev() {
        solution[index] = (right[index] + rho * solution[index + 1]) / diagonal[index];
    }
    Ok(solution)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_a_constant_signal() {
        let smoothed = tvreg_smoothed_series(&[3.0; 8], 0.5, 100).unwrap();
        assert!(smoothed.iter().all(|value| (*value - 3.0).abs() < 1e-8));
    }

    #[test]
    fn removes_small_variation_from_a_step_signal() {
        let smoothed =
            tvreg_smoothed_series(&[0.0, 0.2, -0.1, 0.1, 5.0, 5.1, 4.9, 5.0], 0.7, 250).unwrap();
        assert!(smoothed[0..4].iter().all(|value| value.abs() < 0.5));
        assert!(smoothed[4..].iter().all(|value| (*value - 5.0).abs() < 0.5));
    }

    #[test]
    fn differentiates_a_linear_signal() {
        let time = [0.0, 1.0, 2.0, 3.0, 4.0];
        let values = time.iter().map(|time| 2.0 * time + 1.0).collect::<Vec<_>>();
        let derivative = tvreg_series(&time, &values, 0.1, 100).unwrap();
        // TV regularization has expected endpoint shrinkage; the derivative
        // remains accurate to the denoising scale across the full signal.
        assert!(derivative.iter().all(|value| (*value - 2.0).abs() < 0.11));
    }

    #[test]
    fn rejects_invalid_regularization_settings() {
        assert_eq!(
            tvreg_smoothed_series(&[1.0, 2.0], 0.0, 10),
            Err(DifferentiationError::InvalidTotalVariationConfig)
        );
        assert_eq!(
            tvreg_smoothed_series(&[1.0, 2.0], 0.5, 0),
            Err(DifferentiationError::InvalidTotalVariationConfig)
        );
    }
}
