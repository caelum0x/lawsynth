//! Deterministic configuration for basin mapping.

use lawsynth_stability::StabilityConfig;

use crate::error::BasinError;

/// Default number of initial-condition samples per axis.
pub const DEFAULT_GRID_RESOLUTION: usize = 21;
/// Default fixed RK4 step size for the forward flow.
pub const DEFAULT_DT: f64 = 0.01;
/// Default maximum integration time per initial condition.
pub const DEFAULT_MAX_TIME: f64 = 50.0;
/// Default `‖x − x*‖∞` at which a trajectory is declared converged to `x*`.
pub const DEFAULT_CONVERGENCE_TOLERANCE: f64 = 1e-3;
/// Default padding beyond the search box past which a trajectory is `Escaped`.
pub const DEFAULT_ESCAPE_MARGIN: f64 = 1.0;
/// Default coordinate magnitude past which a trajectory is deemed to diverge.
pub const DEFAULT_DIVERGENCE_LIMIT: f64 = 1e6;

/// Deterministic configuration for [`crate::map_basins`].
///
/// The `search_box` fixes both the initial-condition grid and the escape region
/// (the box padded by `escape_margin`). A separate [`StabilityConfig`] finds the
/// attractors; by default it shares the same box, but it can be overridden so
/// attractor detection and basin sampling use different resolutions.
#[derive(Clone, Debug, PartialEq)]
pub struct BasinConfig {
    search_box: Vec<(f64, f64)>,
    grid_resolution: usize,
    dt: f64,
    max_time: f64,
    convergence_tolerance: f64,
    escape_margin: f64,
    divergence_limit: f64,
    stability: StabilityConfig,
}

impl BasinConfig {
    /// Builds a config over `search_box` (one `(lower, upper)` interval per state)
    /// with default numeric parameters and a matching [`StabilityConfig`] for
    /// attractor detection.
    pub fn new(search_box: Vec<(f64, f64)>) -> Self {
        let stability = StabilityConfig::new(search_box.clone());
        Self {
            search_box,
            grid_resolution: DEFAULT_GRID_RESOLUTION,
            dt: DEFAULT_DT,
            max_time: DEFAULT_MAX_TIME,
            convergence_tolerance: DEFAULT_CONVERGENCE_TOLERANCE,
            escape_margin: DEFAULT_ESCAPE_MARGIN,
            divergence_limit: DEFAULT_DIVERGENCE_LIMIT,
            stability,
        }
    }

    /// Sets the number of initial-condition samples per axis.
    pub fn with_grid_resolution(mut self, grid_resolution: usize) -> Self {
        self.grid_resolution = grid_resolution;
        self
    }

    /// Sets the fixed RK4 step size.
    pub fn with_dt(mut self, dt: f64) -> Self {
        self.dt = dt;
        self
    }

    /// Sets the maximum integration time per initial condition.
    pub fn with_max_time(mut self, max_time: f64) -> Self {
        self.max_time = max_time;
        self
    }

    /// Sets the `‖x − x*‖∞` convergence tolerance.
    pub fn with_convergence_tolerance(mut self, convergence_tolerance: f64) -> Self {
        self.convergence_tolerance = convergence_tolerance;
        self
    }

    /// Sets the padding beyond the search box past which a trajectory escapes.
    pub fn with_escape_margin(mut self, escape_margin: f64) -> Self {
        self.escape_margin = escape_margin;
        self
    }

    /// Sets the coordinate magnitude past which a trajectory is deemed diverged.
    pub fn with_divergence_limit(mut self, divergence_limit: f64) -> Self {
        self.divergence_limit = divergence_limit;
        self
    }

    /// Overrides the [`StabilityConfig`] used to locate the attractors.
    pub fn with_stability_config(mut self, stability: StabilityConfig) -> Self {
        self.stability = stability;
        self
    }

    /// The per-state `(lower, upper)` search intervals.
    pub fn search_box(&self) -> &[(f64, f64)] {
        &self.search_box
    }

    /// The number of initial-condition samples per axis.
    pub fn grid_resolution(&self) -> usize {
        self.grid_resolution
    }

    /// The fixed RK4 step size.
    pub fn dt(&self) -> f64 {
        self.dt
    }

    /// The maximum integration time per initial condition.
    pub fn max_time(&self) -> f64 {
        self.max_time
    }

    /// The convergence tolerance on `‖x − x*‖∞`.
    pub fn convergence_tolerance(&self) -> f64 {
        self.convergence_tolerance
    }

    /// The padding beyond the box past which a trajectory escapes.
    pub fn escape_margin(&self) -> f64 {
        self.escape_margin
    }

    /// The coordinate magnitude past which a trajectory diverges.
    pub fn divergence_limit(&self) -> f64 {
        self.divergence_limit
    }

    /// The [`StabilityConfig`] used to locate the attractors.
    pub fn stability_config(&self) -> &StabilityConfig {
        &self.stability
    }

    /// The number of fixed RK4 steps taken per initial condition.
    pub(crate) fn step_count(&self) -> usize {
        (self.max_time / self.dt).ceil() as usize
    }

    /// Validates the config against a concrete state-space `dimension`.
    pub(crate) fn validate(&self, dimension: usize) -> Result<(), BasinError> {
        if self.search_box.len() != dimension {
            return Err(BasinError::DimensionMismatch {
                states: dimension,
                search_box: self.search_box.len(),
            });
        }
        for (index, &(lower, upper)) in self.search_box.iter().enumerate() {
            if !lower.is_finite() || !upper.is_finite() || lower > upper {
                return Err(BasinError::InvalidSearchInterval { index, lower, upper });
            }
        }
        if self.grid_resolution == 0 {
            return Err(BasinError::InvalidConfig("grid_resolution must be >= 1"));
        }
        if !self.dt.is_finite() || self.dt <= 0.0 {
            return Err(BasinError::InvalidConfig("dt must be finite and > 0"));
        }
        if !self.max_time.is_finite() || self.max_time <= 0.0 {
            return Err(BasinError::InvalidConfig("max_time must be finite and > 0"));
        }
        if !self.convergence_tolerance.is_finite() || self.convergence_tolerance <= 0.0 {
            return Err(BasinError::InvalidConfig("convergence_tolerance must be finite and > 0"));
        }
        if !self.escape_margin.is_finite() || self.escape_margin < 0.0 {
            return Err(BasinError::InvalidConfig("escape_margin must be finite and >= 0"));
        }
        if !self.divergence_limit.is_finite() || self.divergence_limit <= 0.0 {
            return Err(BasinError::InvalidConfig("divergence_limit must be finite and > 0"));
        }
        if self.stability.search_box().len() != self.search_box.len() {
            return Err(BasinError::InvalidConfig(
                "stability search box dimension must match the basin search box",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_overrides_defaults() {
        let config = BasinConfig::new(vec![(-1.0, 1.0)])
            .with_grid_resolution(9)
            .with_dt(0.05)
            .with_max_time(20.0)
            .with_convergence_tolerance(1e-2)
            .with_escape_margin(2.0)
            .with_divergence_limit(1e4);
        assert_eq!(config.grid_resolution(), 9);
        assert_eq!(config.dt(), 0.05);
        assert_eq!(config.max_time(), 20.0);
        assert_eq!(config.convergence_tolerance(), 1e-2);
        assert_eq!(config.escape_margin(), 2.0);
        assert_eq!(config.divergence_limit(), 1e4);
    }

    #[test]
    fn step_count_rounds_up() {
        let config = BasinConfig::new(vec![(-1.0, 1.0)]).with_dt(0.1).with_max_time(1.0);
        assert_eq!(config.step_count(), 10);
        let config = config.with_max_time(0.95);
        assert_eq!(config.step_count(), 10);
    }

    #[test]
    fn rejects_dimension_mismatch() {
        let config = BasinConfig::new(vec![(-1.0, 1.0)]);
        assert_eq!(
            config.validate(2),
            Err(BasinError::DimensionMismatch { states: 2, search_box: 1 })
        );
    }

    #[test]
    fn rejects_inverted_interval() {
        let config = BasinConfig::new(vec![(1.0, -1.0)]);
        assert!(matches!(
            config.validate(1),
            Err(BasinError::InvalidSearchInterval { index: 0, .. })
        ));
    }

    #[test]
    fn rejects_bad_scalars() {
        let base = BasinConfig::new(vec![(-1.0, 1.0)]);
        assert!(matches!(base.clone().with_dt(0.0).validate(1), Err(BasinError::InvalidConfig(_))));
        assert!(matches!(
            base.clone().with_max_time(-1.0).validate(1),
            Err(BasinError::InvalidConfig(_))
        ));
        assert!(matches!(
            base.with_grid_resolution(0).validate(1),
            Err(BasinError::InvalidConfig(_))
        ));
    }
}
