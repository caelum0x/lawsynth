/// Deterministic composite trapezoidal integral of samples `f` over the
/// (possibly irregular) grid `time`.
///
/// The trapezoid rule is used rather than Simpson's because it makes no
/// assumption of a regular grid and sums terms in a single fixed left-to-right
/// order, which keeps the result bit-reproducible. For a smooth integrand its
/// error is `O(h²)`; the integrands here (a compactly-supported analytic bump
/// against the data) are smooth, so a fine sample grid drives the error down
/// quickly. Callers are expected to have validated that the slices are aligned
/// and hold at least two finite samples.
pub(crate) fn trapezoid(time: &[f64], f: &[f64]) -> f64 {
    debug_assert_eq!(time.len(), f.len());
    debug_assert!(time.len() >= 2);
    (0..time.len() - 1)
        .map(|index| 0.5 * (time[index + 1] - time[index]) * (f[index] + f[index + 1]))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integrates_a_constant_exactly() {
        let time: Vec<f64> = (0..11).map(|i| i as f64 * 0.5).collect();
        let f = vec![2.0; time.len()];
        assert!((trapezoid(&time, &f) - 2.0 * 5.0).abs() < 1e-12);
    }

    #[test]
    fn integrates_a_line_exactly() {
        // ∫_0^4 t dt = 8; the trapezoid rule is exact for linear integrands.
        let time: Vec<f64> = (0..5).map(|i| i as f64).collect();
        let f = time.clone();
        assert!((trapezoid(&time, &f) - 8.0).abs() < 1e-12);
    }

    #[test]
    fn handles_an_irregular_grid() {
        let time = vec![0.0, 0.25, 1.0, 3.0];
        let f = vec![1.0, 1.0, 1.0, 1.0];
        assert!((trapezoid(&time, &f) - 3.0).abs() < 1e-12);
    }
}
