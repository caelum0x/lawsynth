//! The Lyapunov spectrum report and its derived diagnostics.

use std::fmt::Write as _;

/// The estimated Lyapunov spectrum of a discovered field, together with the
/// diagnostics that read chaos and attractor dimension off it.
///
/// The `exponents` are sorted **descending**, so `exponents[0]` is the largest
/// (the one whose positivity signals chaos). The `sum` equals the time-averaged
/// divergence (the mean trace of the Jacobian along the trajectory) and is the
/// tightest, most reliable quantity here; individual exponents converge more
/// slowly. `kaplan_yorke_dimension` is the Lyapunov (Kaplan–Yorke) dimension
/// estimate of the attractor.
#[derive(Clone, Debug, PartialEq)]
pub struct LyapunovReport {
    exponents: Vec<f64>,
    largest: f64,
    sum: f64,
    kaplan_yorke_dimension: f64,
    integration_time: f64,
}

impl LyapunovReport {
    /// Builds a report from the descending-sorted exponents and the elapsed
    /// averaging time. Panics only if handed an empty spectrum, which the
    /// integrator never does (an empty state space is rejected upstream).
    pub(crate) fn new(exponents: Vec<f64>, integration_time: f64) -> Self {
        assert!(!exponents.is_empty(), "a spectrum has at least one exponent");
        let largest = exponents[0];
        let sum = exponents.iter().sum();
        let kaplan_yorke_dimension = kaplan_yorke_dimension(&exponents);
        Self { exponents, largest, sum, kaplan_yorke_dimension, integration_time }
    }

    /// The Lyapunov spectrum, sorted descending.
    pub fn exponents(&self) -> &[f64] {
        &self.exponents
    }

    /// The largest Lyapunov exponent (`exponents[0]`). A positive value is the
    /// signature of chaos.
    pub fn largest(&self) -> f64 {
        self.largest
    }

    /// The sum of the exponents — the time-averaged divergence of the flow. For
    /// an autonomous field this equals the mean trace of the Jacobian along the
    /// trajectory and is the tightest quantity in the report.
    pub fn sum(&self) -> f64 {
        self.sum
    }

    /// The Kaplan–Yorke (Lyapunov) dimension estimate of the attractor.
    pub fn kaplan_yorke_dimension(&self) -> f64 {
        self.kaplan_yorke_dimension
    }

    /// The elapsed time over which the exponents were averaged (the post-transient
    /// window, in the same time units as `dt`).
    pub fn integration_time(&self) -> f64 {
        self.integration_time
    }

    /// The number of exponents in the spectrum (the state-space dimension `n`).
    pub fn dimension(&self) -> usize {
        self.exponents.len()
    }

    /// A stable textual fingerprint of the whole report, encoding every float by
    /// its `f64` bit pattern. Two runs on identical input MUST produce identical
    /// strings; this is the basis of the determinism guarantee.
    pub fn to_canonical_string(&self) -> String {
        let mut output = String::new();
        output.push_str("exponents:");
        for value in &self.exponents {
            let _ = write!(output, "{:016x},", value.to_bits());
        }
        let _ = write!(
            output,
            "\nlargest:{:016x}\nsum:{:016x}\nkaplan_yorke:{:016x}\ntime:{:016x}\n",
            self.largest.to_bits(),
            self.sum.to_bits(),
            self.kaplan_yorke_dimension.to_bits(),
            self.integration_time.to_bits(),
        );
        output
    }
}

/// The Kaplan–Yorke (Lyapunov) dimension from a descending-sorted spectrum.
///
/// With `λ_1 ≥ λ_2 ≥ … ≥ λ_n`, let `j` be the largest index whose partial sum
/// `Σ_{i≤j} λ_i` is non-negative. Then
///
/// ```text
/// D_KY = j + (Σ_{i≤j} λ_i) / |λ_{j+1}|.
/// ```
///
/// Boundary cases, reported honestly rather than forced:
/// - if even `λ_1 < 0` (every partial sum negative) there is no expanding
///   direction and the dimension is `0`;
/// - if every partial sum stays non-negative (`j = n`) the fraction is undefined
///   and the full dimension `n` is reported.
pub(crate) fn kaplan_yorke_dimension(exponents: &[f64]) -> f64 {
    let n = exponents.len();

    // Cumulative partial sums; find the largest index whose sum is non-negative.
    let mut running = 0.0;
    let mut cumulative = Vec::with_capacity(n);
    for &lambda in exponents {
        running += lambda;
        cumulative.push(running);
    }

    let mut j = 0; // 1-based count of leading exponents with a non-negative sum
    for (index, &partial) in cumulative.iter().enumerate() {
        if partial >= 0.0 {
            j = index + 1;
        }
    }

    if j == 0 {
        return 0.0;
    }
    if j == n {
        return n as f64;
    }
    // exponents[j] is λ_{j+1} (0-based) and is strictly negative here, since the
    // partial sum turned negative when it was added.
    j as f64 + cumulative[j - 1] / exponents[j].abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-12, "expected {a} ≈ {b}");
    }

    #[test]
    fn all_negative_spectrum_has_zero_dimension() {
        // A stable fixed point: no expanding direction.
        approx(kaplan_yorke_dimension(&[-1.0, -2.0]), 0.0);
    }

    #[test]
    fn all_non_negative_spectrum_reports_full_dimension() {
        approx(kaplan_yorke_dimension(&[0.5, 0.1]), 2.0);
    }

    #[test]
    fn lorenz_like_spectrum_has_fractional_dimension() {
        // λ ≈ {0.906, 0, -14.57} gives the textbook D_KY ≈ 2.062.
        let d = kaplan_yorke_dimension(&[0.906, 0.0, -14.572]);
        assert!((d - 2.062).abs() < 1e-3, "got {d}");
    }

    #[test]
    fn report_exposes_sorted_spectrum_and_sum() {
        let report = LyapunovReport::new(vec![0.9, 0.0, -14.6], 100.0);
        assert_eq!(report.largest(), 0.9);
        approx(report.sum(), -13.7);
        assert_eq!(report.dimension(), 3);
        assert_eq!(report.exponents(), &[0.9, 0.0, -14.6]);
    }

    #[test]
    fn canonical_string_is_stable() {
        let a = LyapunovReport::new(vec![0.9, -1.0], 50.0);
        let b = LyapunovReport::new(vec![0.9, -1.0], 50.0);
        assert_eq!(a.to_canonical_string(), b.to_canonical_string());
    }
}
