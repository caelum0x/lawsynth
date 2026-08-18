use crate::error::SensitivityError;

/// Default integration step used when none is specified.
pub const DEFAULT_STEP: f64 = 1e-2;
/// Default number of integration steps.
pub const DEFAULT_STEPS: usize = 100;
/// Default start time of the integration.
pub const DEFAULT_START: f64 = 0.0;

/// Deterministic configuration for [`crate::forward_sensitivities`].
///
/// The augmented state-and-sensitivity system is advanced with a single
/// fixed-step fourth-order Runge–Kutta integrator, so the whole run is pinned by
/// three numbers: the start time `t0`, the step `dt`, and the number of `steps`.
/// The produced trajectory carries `steps + 1` samples, the first at `t0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SensitivityConfig {
    t0: f64,
    dt: f64,
    steps: usize,
}

impl SensitivityConfig {
    /// Builds a config with an explicit start time, step, and step count.
    pub fn new(t0: f64, dt: f64, steps: usize) -> Self {
        Self { t0, dt, steps }
    }

    /// Sets the integration start time `t0`.
    pub fn with_start(mut self, t0: f64) -> Self {
        self.t0 = t0;
        self
    }

    /// Sets the fixed integration step `dt`.
    pub fn with_step(mut self, dt: f64) -> Self {
        self.dt = dt;
        self
    }

    /// Sets the number of integration steps (the trajectory has `steps + 1`
    /// samples).
    pub fn with_steps(mut self, steps: usize) -> Self {
        self.steps = steps;
        self
    }

    /// The integration start time.
    pub fn start(&self) -> f64 {
        self.t0
    }

    /// The fixed integration step.
    pub fn step(&self) -> f64 {
        self.dt
    }

    /// The number of integration steps.
    pub fn steps(&self) -> usize {
        self.steps
    }

    /// Validates the numeric knobs. The step must be finite and strictly
    /// positive, the start time finite, and at least one step requested.
    pub(crate) fn validate(&self) -> Result<(), SensitivityError> {
        if !self.t0.is_finite() {
            return Err(SensitivityError::InvalidConfig("t0 must be finite"));
        }
        if !self.dt.is_finite() || self.dt <= 0.0 {
            return Err(SensitivityError::InvalidConfig("dt must be finite and > 0"));
        }
        if self.steps == 0 {
            return Err(SensitivityError::InvalidConfig("steps must be >= 1"));
        }
        Ok(())
    }
}

impl Default for SensitivityConfig {
    fn default() -> Self {
        Self { t0: DEFAULT_START, dt: DEFAULT_STEP, steps: DEFAULT_STEPS }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_overrides_defaults() {
        let config = SensitivityConfig::default().with_start(1.0).with_step(0.05).with_steps(42);
        assert_eq!(config.start(), 1.0);
        assert_eq!(config.step(), 0.05);
        assert_eq!(config.steps(), 42);
    }

    #[test]
    fn rejects_non_positive_step() {
        assert!(matches!(
            SensitivityConfig::new(0.0, 0.0, 10).validate(),
            Err(SensitivityError::InvalidConfig(_))
        ));
        assert!(matches!(
            SensitivityConfig::new(0.0, -0.1, 10).validate(),
            Err(SensitivityError::InvalidConfig(_))
        ));
    }

    #[test]
    fn rejects_zero_steps() {
        assert!(matches!(
            SensitivityConfig::new(0.0, 0.1, 0).validate(),
            Err(SensitivityError::InvalidConfig(_))
        ));
    }

    #[test]
    fn rejects_non_finite_start() {
        assert!(matches!(
            SensitivityConfig::new(f64::NAN, 0.1, 10).validate(),
            Err(SensitivityError::InvalidConfig(_))
        ));
    }
}
