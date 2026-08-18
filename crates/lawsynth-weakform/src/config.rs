use crate::WeakError;

/// Configuration for weak / integral-form discovery.
///
/// Every field is an explicit, deterministic control: test-function placement,
/// radius, and quadrature are all fixed by these values, so identical inputs
/// and configuration produce bit-identical output. There is no randomness.
#[derive(Clone, Debug, PartialEq)]
pub struct WeakConfig {
    /// Number of test functions `K` placed across the usable time window.
    pub test_function_count: usize,
    /// Half-width of each test-function support as a fraction of the time span,
    /// in the open interval `(0, 1)`. The absolute radius is
    /// `support_fraction * (t_end - t_start) / 2`.
    pub support_fraction: f64,
    /// Polynomial order `p` of the bump `φ(t) = (1 - s²)^p`. Must be `>= 2` so
    /// that both `φ` and `φ̇` vanish at the support boundary.
    pub test_function_order: usize,
    /// Maximum total degree of the polynomial candidate library.
    pub feature_degree: usize,
    /// Whether the candidate library includes the constant term.
    pub include_constant: bool,
    /// Sequentially-thresholded least-squares sparsity threshold: coefficients
    /// below this magnitude are pruned between refits.
    pub threshold: f64,
    /// Tikhonov (ridge) regularisation added to the normal-equations diagonal.
    pub ridge: f64,
    /// Maximum number of thresholding iterations.
    pub max_iterations: usize,
}

impl Default for WeakConfig {
    fn default() -> Self {
        Self {
            test_function_count: 16,
            support_fraction: 0.3,
            test_function_order: 4,
            feature_degree: 2,
            include_constant: true,
            threshold: 0.05,
            ridge: 1e-8,
            max_iterations: 12,
        }
    }
}

impl WeakConfig {
    pub(crate) fn validate(&self) -> Result<(), WeakError> {
        if self.test_function_count == 0 {
            return Err(WeakError::NoTestFunctions);
        }
        if self.test_function_order < 2 {
            return Err(WeakError::OrderTooLow { order: self.test_function_order });
        }
        if !self.support_fraction.is_finite()
            || self.support_fraction <= 0.0
            || self.support_fraction >= 1.0
        {
            return Err(WeakError::InvalidSupportFraction { value: self.support_fraction });
        }
        if !self.threshold.is_finite() || self.threshold < 0.0 {
            return Err(WeakError::InvalidConfig { field: "threshold" });
        }
        if !self.ridge.is_finite() || self.ridge < 0.0 {
            return Err(WeakError::InvalidConfig { field: "ridge" });
        }
        if self.max_iterations == 0 {
            return Err(WeakError::InvalidConfig { field: "max_iterations" });
        }
        Ok(())
    }
}
