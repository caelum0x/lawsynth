use crate::ReduceError;

/// Deterministic controls for structural-reduction detection.
///
/// Every field has a documented default. Tolerances are the screening /
/// verification thresholds described in `specs/structural-reductions/README.md`;
/// widening them trades false negatives for false positives, so the defaults are
/// deliberately tight.
#[derive(Clone, Debug, PartialEq)]
pub struct ReduceConfig {
    /// The column treated as the scalar target `f`. When `None`, the
    /// lexicographically greatest column id in the (sorted) schema is used.
    pub target: Option<String>,
    /// Maximum normalized mixed-partial screening residual for a separability to
    /// pass the screen.
    pub separability_screen_tol: f64,
    /// Maximum relative reconstruction residual (`1 − R²`) for an *additive*
    /// separability to be reported.
    pub additive_tol: f64,
    /// Maximum relative reconstruction residual for a *multiplicative*
    /// separability to be reported.
    pub multiplicative_tol: f64,
    /// Maximum normalized first-derivative invariance residual for a symmetry to
    /// be reported.
    pub symmetry_tol: f64,
    /// Maximum number of input variables (guards partition enumeration and grid
    /// dimensionality).
    pub max_variables: usize,
    /// Minimum distinct values required per grid axis (needs `≥ 3` for a stable
    /// mixed second partial).
    pub min_axis_len: usize,
    /// Relative tolerance used to merge near-equal coordinate values into a
    /// single grid axis level.
    pub grid_dedup_rel_tol: f64,
    /// A field whose value range is at or below this (relative to its mean
    /// magnitude) is treated as constant and skipped.
    pub constant_field_tol: f64,
    /// `|f|` must exceed this on every grid node for the multiplicative
    /// (log-domain) analysis to be attempted.
    pub multiplicative_floor: f64,
}

impl Default for ReduceConfig {
    fn default() -> Self {
        Self {
            target: None,
            separability_screen_tol: 1.0e-2,
            additive_tol: 1.0e-3,
            multiplicative_tol: 1.0e-3,
            symmetry_tol: 1.0e-2,
            max_variables: 6,
            min_axis_len: 3,
            grid_dedup_rel_tol: 1.0e-9,
            constant_field_tol: 1.0e-12,
            multiplicative_floor: 1.0e-9,
        }
    }
}

impl ReduceConfig {
    /// Convenience constructor that sets the target column explicitly.
    pub fn with_target(target: impl Into<String>) -> Self {
        Self { target: Some(target.into()), ..Self::default() }
    }

    /// Validates finite, in-range configuration values.
    pub(crate) fn validate(&self) -> Result<(), ReduceError> {
        let positive = [
            ("separability_screen_tol", self.separability_screen_tol),
            ("additive_tol", self.additive_tol),
            ("multiplicative_tol", self.multiplicative_tol),
            ("symmetry_tol", self.symmetry_tol),
            ("grid_dedup_rel_tol", self.grid_dedup_rel_tol),
            ("constant_field_tol", self.constant_field_tol),
            ("multiplicative_floor", self.multiplicative_floor),
        ];
        for (field, value) in positive {
            if !value.is_finite() || value <= 0.0 {
                return Err(ReduceError::InvalidConfig { field });
            }
        }
        if self.max_variables == 0 {
            return Err(ReduceError::InvalidConfig { field: "max_variables" });
        }
        if self.min_axis_len < 3 {
            return Err(ReduceError::InvalidConfig { field: "min_axis_len" });
        }
        Ok(())
    }
}
