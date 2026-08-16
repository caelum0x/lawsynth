use crate::PreprocessError;

/// Shared guardrails for constructing deterministic preprocessing pipelines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreprocessConfig {
    pub maximum_steps: usize,
}
impl Default for PreprocessConfig {
    fn default() -> Self {
        Self { maximum_steps: 32 }
    }
}
impl PreprocessConfig {
    pub fn validate_steps(self, steps: usize) -> Result<(), PreprocessError> {
        if self.maximum_steps == 0 || steps > self.maximum_steps {
            Err(PreprocessError::InvalidTargetTime)
        } else {
            Ok(())
        }
    }
}
