use crate::WeakError;

/// A compactly-supported polynomial bump test function `φ` and its analytic
/// derivative `φ̇`, centred at `center` with half-width `radius`.
///
/// `φ(t) = (1 - s²)^p` for `s = (t - center) / radius` on `|s| < 1`, and `0`
/// outside. With order `p >= 2` both `φ` and `φ̇` are continuous and vanish at
/// `s = ±1`, so integration by parts drops all boundary terms. The derivative
/// is evaluated from the closed form, never from the (noisy) data — that is the
/// entire point of the weak formulation.
#[derive(Clone, Debug, PartialEq)]
pub struct TestFunction {
    center: f64,
    radius: f64,
    order: usize,
}

impl TestFunction {
    pub fn new(center: f64, radius: f64, order: usize) -> Self {
        Self { center, radius, order }
    }

    pub fn center(&self) -> f64 {
        self.center
    }

    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// `φ(t) = (1 - s²)^p`, zero outside the support.
    pub fn value(&self, t: f64) -> f64 {
        let s = (t - self.center) / self.radius;
        if s.abs() >= 1.0 { 0.0 } else { (1.0 - s * s).powi(self.order as i32) }
    }

    /// `φ̇(t) = -(2p / r) · s · (1 - s²)^{p-1}`, zero outside the support.
    pub fn derivative(&self, t: f64) -> f64 {
        let s = (t - self.center) / self.radius;
        if s.abs() >= 1.0 {
            0.0
        } else {
            let p = self.order as f64;
            -(2.0 * p / self.radius) * s * (1.0 - s * s).powi(self.order as i32 - 1)
        }
    }

    /// Number of grid samples that fall strictly inside the support.
    fn samples_in_support(&self, time: &[f64]) -> usize {
        time.iter().filter(|&&t| ((t - self.center) / self.radius).abs() < 1.0).count()
    }
}

/// Places `count` test functions on evenly spaced, content-derived centres
/// inside the usable window `[t0 + r, t_end - r]`.
///
/// Placement is fully deterministic: centres come only from the time bounds and
/// the requested count — no randomness, no wall-clock. Each support is validated
/// to contain at least two grid samples so its quadrature is well posed.
pub fn place(
    time: &[f64],
    count: usize,
    support_fraction: f64,
    order: usize,
) -> Result<Vec<TestFunction>, WeakError> {
    let first = time[0];
    let last = time[time.len() - 1];
    let span = last - first;
    let radius = support_fraction * span / 2.0;
    if !(radius.is_finite() && radius > 0.0) {
        return Err(WeakError::InvalidSupportFraction { value: support_fraction });
    }
    let usable_lo = first + radius;
    let usable_hi = last - radius;

    let functions = (0..count)
        .map(|index| {
            let center = if count == 1 {
                0.5 * (first + last)
            } else {
                usable_lo + (usable_hi - usable_lo) * index as f64 / (count - 1) as f64
            };
            TestFunction::new(center, radius, order)
        })
        .collect::<Vec<_>>();

    for function in &functions {
        if function.samples_in_support(time) < 2 {
            return Err(WeakError::EmptySupport {
                center: function.center,
                radius: function.radius,
            });
        }
    }
    Ok(functions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bump_and_derivative_vanish_at_the_boundary() {
        let phi = TestFunction::new(0.0, 1.0, 4);
        assert_eq!(phi.value(1.0), 0.0);
        assert_eq!(phi.value(-1.0), 0.0);
        assert_eq!(phi.derivative(1.0), 0.0);
        assert_eq!(phi.derivative(-1.0), 0.0);
        assert_eq!(phi.value(0.0), 1.0);
        assert_eq!(phi.derivative(0.0), 0.0);
    }

    #[test]
    fn derivative_matches_a_finite_difference_of_the_value() {
        let phi = TestFunction::new(0.3, 0.7, 5);
        let h = 1e-6;
        for &t in &[-0.1, 0.0, 0.25, 0.5] {
            let numeric = (phi.value(t + h) - phi.value(t - h)) / (2.0 * h);
            assert!((numeric - phi.derivative(t)).abs() < 1e-4, "t = {t}");
        }
    }

    #[test]
    fn places_evenly_spaced_centres_inside_the_usable_window() {
        let time: Vec<f64> = (0..101).map(|i| i as f64 * 0.1).collect();
        let functions = place(&time, 3, 0.4, 4).unwrap();
        assert_eq!(functions.len(), 3);
        let radius = 0.4 * 10.0 / 2.0;
        assert!((functions[0].center() - radius).abs() < 1e-12);
        assert!((functions[2].center() - (10.0 - radius)).abs() < 1e-12);
        assert!((functions[1].center() - 5.0).abs() < 1e-12);
    }

    #[test]
    fn single_test_function_sits_at_the_midpoint() {
        let time: Vec<f64> = (0..21).map(|i| i as f64 * 0.5).collect();
        let functions = place(&time, 1, 0.5, 2).unwrap();
        assert_eq!(functions.len(), 1);
        assert!((functions[0].center() - 5.0).abs() < 1e-12);
    }

    #[test]
    fn rejects_a_support_with_too_few_samples() {
        let time = vec![0.0, 5.0, 10.0];
        // A tiny radius means the outer supports catch no interior samples.
        let result = place(&time, 3, 0.02, 4);
        assert!(matches!(result, Err(WeakError::EmptySupport { .. })));
    }
}
