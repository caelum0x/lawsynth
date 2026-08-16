use std::f64::consts::TAU;

use crate::DifferentiationError;

/// Estimates a derivative with a direct discrete Fourier transform.
///
/// This is an `O(n²)` reference implementation intended for periodic signals
/// and modest sample sizes. It deliberately rejects irregular grids rather than
/// silently applying an FFT formula with invalid frequency spacing.
pub fn spectral_derivative(time: &[f64], values: &[f64]) -> Result<Vec<f64>, DifferentiationError> {
    if time.len() != values.len() {
        return Err(DifferentiationError::LengthMismatch);
    }
    if time.len() < 3 {
        return Err(DifferentiationError::TooFewSamples);
    }
    if values.iter().any(|value| !value.is_finite()) {
        return Err(DifferentiationError::SingularFit);
    }
    let step = time[1] - time[0];
    if !step.is_finite()
        || step <= 0.0
        || time.windows(2).any(|pair| {
            !pair[0].is_finite()
                || !pair[1].is_finite()
                || ((pair[1] - pair[0]) - step).abs() > step.abs().max(1.0) * 1e-10
        })
    {
        return Err(DifferentiationError::IrregularTimeAxis);
    }
    let count = values.len();
    let spectrum = (0..count)
        .map(|frequency| {
            values
                .iter()
                .enumerate()
                .fold((0.0, 0.0), |(real, imaginary), (index, value)| {
                    let angle = TAU * frequency as f64 * index as f64 / count as f64;
                    (real + value * angle.cos(), imaginary - value * angle.sin())
                })
        })
        .collect::<Vec<_>>();
    Ok((0..count)
        .map(|index| {
            spectrum
                .iter()
                .enumerate()
                .map(|(frequency, (real, imaginary))| {
                    let signed_frequency = if frequency <= count / 2 {
                        frequency as isize
                    } else {
                        frequency as isize - count as isize
                    } as f64;
                    let angular_frequency = TAU * signed_frequency / (count as f64 * step);
                    let angle = TAU * frequency as f64 * index as f64 / count as f64;
                    -angular_frequency * (imaginary * angle.cos() + real * angle.sin())
                })
                .sum::<f64>()
                / count as f64
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use std::f64::consts::TAU;

    use super::*;

    #[test]
    fn differentiates_a_periodic_sine_wave() {
        let count = 64;
        let time = (0..count)
            .map(|index| index as f64 / count as f64)
            .collect::<Vec<_>>();
        let values = time
            .iter()
            .map(|time| (TAU * time).sin())
            .collect::<Vec<_>>();
        let derivative = spectral_derivative(&time, &values).unwrap();
        assert!(
            derivative
                .iter()
                .zip(&time)
                .all(|(actual, time)| (*actual - TAU * (TAU * time).cos()).abs() < 1e-10)
        );
    }
}
