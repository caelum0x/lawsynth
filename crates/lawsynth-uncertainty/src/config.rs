use crate::UncertaintyError;

/// Controls deterministic non-parametric bootstrapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BootstrapConfig {
    pub replicates: usize,
    pub seed: u64,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            replicates: 1_000,
            seed: 0x4c_4157_5359_4e54,
        }
    }
}

impl BootstrapConfig {
    pub fn validate(self) -> Result<(), UncertaintyError> {
        if self.replicates == 0 {
            Err(UncertaintyError::InvalidBootstrapConfig)
        } else {
            Ok(())
        }
    }
}

/// Defines a two-sided central confidence interval.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntervalConfig {
    pub confidence: f64,
}

impl Default for IntervalConfig {
    fn default() -> Self {
        Self { confidence: 0.95 }
    }
}

impl IntervalConfig {
    pub fn validate(self) -> Result<(), UncertaintyError> {
        if self.confidence.is_finite() && self.confidence > 0.0 && self.confidence < 1.0 {
            Ok(())
        } else {
            Err(UncertaintyError::InvalidConfidence(self.confidence))
        }
    }
}

/// Controls deterministic Monte-Carlo propagation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PropagationConfig {
    pub draws: usize,
    pub seed: u64,
}

impl Default for PropagationConfig {
    fn default() -> Self {
        Self {
            draws: 10_000,
            seed: 0x5052_4f50_4147_4154,
        }
    }
}

impl PropagationConfig {
    pub fn validate(self) -> Result<(), UncertaintyError> {
        if self.draws == 0 {
            Err(UncertaintyError::InvalidPropagationConfig)
        } else {
            Ok(())
        }
    }
}
