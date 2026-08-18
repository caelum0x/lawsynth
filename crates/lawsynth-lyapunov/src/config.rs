use crate::error::LyapunovError;

/// Default integration step.
pub const DEFAULT_STEP: f64 = 1e-2;
/// Default number of integration steps.
pub const DEFAULT_STEPS: usize = 10_000;
/// Default reorthonormalization interval (steps between QR renormalizations).
pub const DEFAULT_REORTH_INTERVAL: usize = 10;
/// Default fraction of the run discarded as transient before averaging begins.
pub const DEFAULT_TRANSIENT_FRACTION: f64 = 0.1;

/// Deterministic configuration for [`crate::lyapunov_spectrum`].
///
/// The Benettin/QR estimator is pinned by four numbers: the fixed integration
/// step `dt`, the number of `steps`, the reorthonormalization interval `k` (how
/// many steps elapse between Gram–Schmidt renormalizations of the perturbation
/// frame), and the `transient_fraction` of the run discarded before the
/// exponents are averaged. Longer runs and smaller steps sharpen the estimate;
/// see the crate docs and `specs/lyapunov-exponents/README.md` for the honest
/// accuracy limits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LyapunovConfig {
    dt: f64,
    steps: usize,
    reorthonormalization_interval: usize,
    transient_fraction: f64,
}

impl LyapunovConfig {
    /// Builds a config with an explicit step, step count, reorthonormalization
    /// interval, and transient fraction.
    pub fn new(
        dt: f64,
        steps: usize,
        reorthonormalization_interval: usize,
        transient_fraction: f64,
    ) -> Self {
        Self { dt, steps, reorthonormalization_interval, transient_fraction }
    }

    /// Sets the fixed integration step `dt`.
    pub fn with_step(mut self, dt: f64) -> Self {
        self.dt = dt;
        self
    }

    /// Sets the number of integration steps.
    pub fn with_steps(mut self, steps: usize) -> Self {
        self.steps = steps;
        self
    }

    /// Sets the reorthonormalization interval `k` (steps between QR renormalizations).
    pub fn with_reorthonormalization_interval(mut self, interval: usize) -> Self {
        self.reorthonormalization_interval = interval;
        self
    }

    /// Sets the transient fraction discarded before averaging (in `[0, 1)`).
    pub fn with_transient_fraction(mut self, fraction: f64) -> Self {
        self.transient_fraction = fraction;
        self
    }

    /// The fixed integration step.
    pub fn step(&self) -> f64 {
        self.dt
    }

    /// The number of integration steps.
    pub fn steps(&self) -> usize {
        self.steps
    }

    /// The reorthonormalization interval `k`.
    pub fn reorthonormalization_interval(&self) -> usize {
        self.reorthonormalization_interval
    }

    /// The transient fraction discarded before averaging.
    pub fn transient_fraction(&self) -> f64 {
        self.transient_fraction
    }

    /// The number of leading steps discarded as transient, `floor(f · steps)`.
    pub(crate) fn transient_steps(&self) -> usize {
        (self.transient_fraction * self.steps as f64).floor() as usize
    }

    /// Validates the numeric knobs. The step must be finite and strictly
    /// positive, at least one step and an interval of at least one are required,
    /// and the transient fraction must be a finite value in `[0, 1)`.
    pub(crate) fn validate(&self) -> Result<(), LyapunovError> {
        if !self.dt.is_finite() || self.dt <= 0.0 {
            return Err(LyapunovError::InvalidConfig("dt must be finite and > 0"));
        }
        if self.steps == 0 {
            return Err(LyapunovError::InvalidConfig("steps must be >= 1"));
        }
        if self.reorthonormalization_interval == 0 {
            return Err(LyapunovError::InvalidConfig("reorthonormalization interval must be >= 1"));
        }
        if !self.transient_fraction.is_finite()
            || self.transient_fraction < 0.0
            || self.transient_fraction >= 1.0
        {
            return Err(LyapunovError::InvalidConfig(
                "transient fraction must be finite and in [0, 1)",
            ));
        }
        Ok(())
    }
}

impl Default for LyapunovConfig {
    fn default() -> Self {
        Self {
            dt: DEFAULT_STEP,
            steps: DEFAULT_STEPS,
            reorthonormalization_interval: DEFAULT_REORTH_INTERVAL,
            transient_fraction: DEFAULT_TRANSIENT_FRACTION,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_overrides_defaults() {
        let config = LyapunovConfig::default()
            .with_step(0.005)
            .with_steps(500)
            .with_reorthonormalization_interval(5)
            .with_transient_fraction(0.25);
        assert_eq!(config.step(), 0.005);
        assert_eq!(config.steps(), 500);
        assert_eq!(config.reorthonormalization_interval(), 5);
        assert_eq!(config.transient_fraction(), 0.25);
    }

    #[test]
    fn transient_steps_is_floor_of_fraction() {
        let config = LyapunovConfig::default().with_steps(1000).with_transient_fraction(0.3);
        assert_eq!(config.transient_steps(), 300);
    }

    #[test]
    fn rejects_non_positive_step() {
        assert!(matches!(
            LyapunovConfig::new(0.0, 10, 1, 0.0).validate(),
            Err(LyapunovError::InvalidConfig(_))
        ));
        assert!(matches!(
            LyapunovConfig::new(-0.1, 10, 1, 0.0).validate(),
            Err(LyapunovError::InvalidConfig(_))
        ));
    }

    #[test]
    fn rejects_zero_steps() {
        assert!(matches!(
            LyapunovConfig::new(0.1, 0, 1, 0.0).validate(),
            Err(LyapunovError::InvalidConfig(_))
        ));
    }

    #[test]
    fn rejects_zero_interval() {
        assert!(matches!(
            LyapunovConfig::new(0.1, 10, 0, 0.0).validate(),
            Err(LyapunovError::InvalidConfig(_))
        ));
    }

    #[test]
    fn rejects_out_of_range_transient() {
        assert!(matches!(
            LyapunovConfig::new(0.1, 10, 1, 1.0).validate(),
            Err(LyapunovError::InvalidConfig(_))
        ));
        assert!(matches!(
            LyapunovConfig::new(0.1, 10, 1, -0.1).validate(),
            Err(LyapunovError::InvalidConfig(_))
        ));
    }
}
