use crate::error::StabilityError;

/// Default number of samples per axis in the seed lattice.
pub const DEFAULT_GRID_RESOLUTION: usize = 5;
/// Default maximum Newton iterations from a single seed.
pub const DEFAULT_MAX_ITERATIONS: usize = 100;
/// Default residual (∞-norm of `f(x)`) at which Newton is declared converged.
pub const DEFAULT_TOLERANCE: f64 = 1e-10;
/// Default radius within which two roots are treated as one fixed point.
pub const DEFAULT_DEDUP_TOLERANCE: f64 = 1e-6;
/// Default half-width of the band around the imaginary axis in which an
/// eigenvalue's real part is treated as zero (linearization inconclusive).
pub const DEFAULT_MARGINAL_BAND: f64 = 1e-6;
/// Default magnitude beyond which a Newton iterate is considered to have
/// diverged and its seed is dropped.
pub const DEFAULT_DIVERGENCE_LIMIT: f64 = 1e6;

/// Deterministic configuration for [`crate::analyze_stability`].
///
/// The search box fixes both where Newton starts (a content-independent lattice
/// plus the origin) and which located roots are reported (roots outside the box
/// are dropped). All numeric knobs are explicit so a run is fully reproducible.
#[derive(Clone, Debug, PartialEq)]
pub struct StabilityConfig {
    search_box: Vec<(f64, f64)>,
    grid_resolution: usize,
    max_iterations: usize,
    tolerance: f64,
    dedup_tolerance: f64,
    marginal_band: f64,
    divergence_limit: f64,
}

impl StabilityConfig {
    /// Builds a config over `search_box` (one `(lower, upper)` interval per state)
    /// with default numeric parameters. The box dimension is validated against
    /// the state count when the analysis runs.
    pub fn new(search_box: Vec<(f64, f64)>) -> Self {
        Self {
            search_box,
            grid_resolution: DEFAULT_GRID_RESOLUTION,
            max_iterations: DEFAULT_MAX_ITERATIONS,
            tolerance: DEFAULT_TOLERANCE,
            dedup_tolerance: DEFAULT_DEDUP_TOLERANCE,
            marginal_band: DEFAULT_MARGINAL_BAND,
            divergence_limit: DEFAULT_DIVERGENCE_LIMIT,
        }
    }

    /// Sets the number of samples per axis in the seed lattice.
    pub fn with_grid_resolution(mut self, grid_resolution: usize) -> Self {
        self.grid_resolution = grid_resolution;
        self
    }

    /// Sets the maximum Newton iterations attempted from a single seed.
    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    /// Sets the Newton convergence tolerance on the residual `‖f(x)‖∞`.
    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// Sets the radius within which two discovered roots are merged.
    pub fn with_dedup_tolerance(mut self, dedup_tolerance: f64) -> Self {
        self.dedup_tolerance = dedup_tolerance;
        self
    }

    /// Sets the half-width of the "real part ≈ 0" band used in classification.
    pub fn with_marginal_band(mut self, marginal_band: f64) -> Self {
        self.marginal_band = marginal_band;
        self
    }

    /// Sets the magnitude beyond which a Newton iterate is deemed diverged.
    pub fn with_divergence_limit(mut self, divergence_limit: f64) -> Self {
        self.divergence_limit = divergence_limit;
        self
    }

    /// The per-state `(lower, upper)` search intervals.
    pub fn search_box(&self) -> &[(f64, f64)] {
        &self.search_box
    }

    /// The number of samples per axis in the seed lattice.
    pub fn grid_resolution(&self) -> usize {
        self.grid_resolution
    }

    /// The maximum Newton iterations per seed.
    pub fn max_iterations(&self) -> usize {
        self.max_iterations
    }

    /// The Newton residual tolerance.
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }

    /// The root-merging radius.
    pub fn dedup_tolerance(&self) -> f64 {
        self.dedup_tolerance
    }

    /// The classification "≈0" band half-width.
    pub fn marginal_band(&self) -> f64 {
        self.marginal_band
    }

    /// The Newton divergence magnitude.
    pub fn divergence_limit(&self) -> f64 {
        self.divergence_limit
    }

    /// Validates the config against a concrete state-space `dimension`.
    pub(crate) fn validate(&self, dimension: usize) -> Result<(), StabilityError> {
        if self.search_box.len() != dimension {
            return Err(StabilityError::DimensionMismatch {
                states: dimension,
                search_box: self.search_box.len(),
            });
        }
        for (index, &(lower, upper)) in self.search_box.iter().enumerate() {
            if !lower.is_finite() || !upper.is_finite() || lower > upper {
                return Err(StabilityError::InvalidSearchInterval { index, lower, upper });
            }
        }
        if self.grid_resolution == 0 {
            return Err(StabilityError::InvalidConfig("grid_resolution must be >= 1"));
        }
        if self.max_iterations == 0 {
            return Err(StabilityError::InvalidConfig("max_iterations must be >= 1"));
        }
        if !self.tolerance.is_finite() || self.tolerance <= 0.0 {
            return Err(StabilityError::InvalidConfig("tolerance must be finite and > 0"));
        }
        if !self.dedup_tolerance.is_finite() || self.dedup_tolerance < 0.0 {
            return Err(StabilityError::InvalidConfig("dedup_tolerance must be finite and >= 0"));
        }
        if !self.marginal_band.is_finite() || self.marginal_band < 0.0 {
            return Err(StabilityError::InvalidConfig("marginal_band must be finite and >= 0"));
        }
        if !self.divergence_limit.is_finite() || self.divergence_limit <= 0.0 {
            return Err(StabilityError::InvalidConfig("divergence_limit must be finite and > 0"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_overrides_defaults() {
        let config = StabilityConfig::new(vec![(-1.0, 1.0)])
            .with_grid_resolution(9)
            .with_max_iterations(50)
            .with_tolerance(1e-8)
            .with_dedup_tolerance(1e-4)
            .with_marginal_band(1e-3)
            .with_divergence_limit(1e4);
        assert_eq!(config.grid_resolution(), 9);
        assert_eq!(config.max_iterations(), 50);
        assert_eq!(config.tolerance(), 1e-8);
        assert_eq!(config.dedup_tolerance(), 1e-4);
        assert_eq!(config.marginal_band(), 1e-3);
        assert_eq!(config.divergence_limit(), 1e4);
    }

    #[test]
    fn rejects_dimension_mismatch() {
        let config = StabilityConfig::new(vec![(-1.0, 1.0)]);
        assert_eq!(
            config.validate(2),
            Err(StabilityError::DimensionMismatch { states: 2, search_box: 1 })
        );
    }

    #[test]
    fn rejects_inverted_interval() {
        let config = StabilityConfig::new(vec![(1.0, -1.0)]);
        assert!(matches!(
            config.validate(1),
            Err(StabilityError::InvalidSearchInterval { index: 0, .. })
        ));
    }

    #[test]
    fn rejects_bad_scalars() {
        let base = StabilityConfig::new(vec![(-1.0, 1.0)]);
        assert!(matches!(
            base.clone().with_grid_resolution(0).validate(1),
            Err(StabilityError::InvalidConfig(_))
        ));
        assert!(matches!(
            base.clone().with_tolerance(0.0).validate(1),
            Err(StabilityError::InvalidConfig(_))
        ));
        assert!(matches!(
            base.with_max_iterations(0).validate(1),
            Err(StabilityError::InvalidConfig(_))
        ));
    }
}
