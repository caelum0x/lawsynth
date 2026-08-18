//! Small deterministic statistics helpers: empirical quantiles and the standard
//! normal quantile used to turn a confidence level into a band multiplier.

use crate::error::PropagateError;

/// Linearly interpolated empirical quantile (the R type-7 rule), matching the
/// convention used by `lawsynth-uncertainty::percentile`.
///
/// `values` is treated as a sample; it is copied into a total ordering (so `NaN`
/// never corrupts the sort) and the quantile at rank `probability·(n − 1)` is
/// interpolated between the two nearest order statistics. `probability` must lie
/// in `[0, 1]` and `values` must be non-empty.
pub(crate) fn percentile(values: &[f64], probability: f64) -> f64 {
    debug_assert!(!values.is_empty());
    debug_assert!((0.0..=1.0).contains(&probability));
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let rank = probability * (sorted.len() - 1) as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    sorted[lower] + (sorted[upper] - sorted[lower]) * (rank - lower as f64)
}

/// The band multiplier `z` for a two-sided `confidence` level: the standard
/// normal quantile at `1 − (1 − confidence)/2`.
///
/// For example `confidence = 0.95` yields `z ≈ 1.959964`. `confidence` must lie
/// in the open interval `(0, 1)`.
///
/// # Errors
///
/// Returns [`PropagateError::InvalidConfidence`] if `confidence ∉ (0, 1)`.
pub fn z_for_confidence(confidence: f64) -> Result<f64, PropagateError> {
    if !confidence.is_finite() || confidence <= 0.0 || confidence >= 1.0 {
        return Err(PropagateError::InvalidConfidence(confidence));
    }
    let upper_tail_probability = 1.0 - (1.0 - confidence) / 2.0;
    Ok(inverse_standard_normal_cdf(upper_tail_probability))
}

/// Acklam's rational approximation to the inverse standard normal CDF.
///
/// Accurate to a relative error below `1.15e-9` across `(0, 1)`; deterministic
/// and dependency-free. `probability` is assumed to lie in `(0, 1)`.
// The constants are Acklam's published coefficients, kept verbatim.
#[allow(clippy::excessive_precision)]
fn inverse_standard_normal_cdf(probability: f64) -> f64 {
    const A: [f64; 6] = [
        -3.969683028665376e+01,
        2.209460984245205e+02,
        -2.759285104469687e+02,
        1.383577518672690e+02,
        -3.066479806614716e+01,
        2.506628277459239e+00,
    ];
    const B: [f64; 5] = [
        -5.447609879822406e+01,
        1.615858368580409e+02,
        -1.556989798598866e+02,
        6.680131188771972e+01,
        -1.328068155288572e+01,
    ];
    const C: [f64; 6] = [
        -7.784894002430293e-03,
        -3.223964580411365e-01,
        -2.400758277161838e+00,
        -2.549732539343734e+00,
        4.374664141464968e+00,
        2.938163982698783e+00,
    ];
    const D: [f64; 4] = [
        7.784695709041462e-03,
        3.224671290700398e-01,
        2.445134137142996e+00,
        3.754408661907416e+00,
    ];
    const P_LOW: f64 = 0.02425;
    const P_HIGH: f64 = 1.0 - P_LOW;

    if probability < P_LOW {
        let q = (-2.0 * probability.ln()).sqrt();
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    } else if probability <= P_HIGH {
        let q = probability - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - probability).ln()).sqrt();
        -(((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_matches_r_type_seven() {
        let values = [0.0, 1.0, 2.0, 3.0, 4.0];
        // probability 0.1 -> rank 0.4 -> 0 + 0.4*(1-0) = 0.4.
        assert!((percentile(&values, 0.1) - 0.4).abs() < 1e-12);
        // probability 0.9 -> rank 3.6 -> 3 + 0.6*(4-3) = 3.6.
        assert!((percentile(&values, 0.9) - 3.6).abs() < 1e-12);
        // Median is the middle order statistic.
        assert_eq!(percentile(&values, 0.5), 2.0);
    }

    #[test]
    fn z_for_confidence_matches_known_quantiles() {
        assert!((z_for_confidence(0.95).unwrap() - 1.959963985).abs() < 1e-6);
        assert!((z_for_confidence(0.99).unwrap() - 2.575829304).abs() < 1e-6);
        assert!((z_for_confidence(0.6826894921).unwrap() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn z_for_confidence_rejects_out_of_range() {
        assert_eq!(z_for_confidence(0.0), Err(PropagateError::InvalidConfidence(0.0)));
        assert_eq!(z_for_confidence(1.0), Err(PropagateError::InvalidConfidence(1.0)));
        assert!(matches!(z_for_confidence(f64::NAN), Err(PropagateError::InvalidConfidence(_))));
    }

    #[test]
    fn higher_confidence_gives_larger_multiplier() {
        let z90 = z_for_confidence(0.90).unwrap();
        let z95 = z_for_confidence(0.95).unwrap();
        let z99 = z_for_confidence(0.99).unwrap();
        assert!(z90 < z95 && z95 < z99);
    }
}
