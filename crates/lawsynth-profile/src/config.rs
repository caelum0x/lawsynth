use crate::ProfileError;

/// Tuning parameters for deterministic input profiling.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProfileConfig {
    /// Relative tolerance used to classify a timestamp axis as regularly sampled.
    pub regularity_tolerance: f64,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            regularity_tolerance: 1e-9,
        }
    }
}

impl ProfileConfig {
    pub fn validate(self) -> Result<Self, ProfileError> {
        if !self.regularity_tolerance.is_finite() || self.regularity_tolerance < 0.0 {
            return Err(ProfileError::InvalidConfiguration);
        }
        Ok(self)
    }
}
