use crate::DynamicsError;

/// Validation limits shared by dataset-backed identification problems.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DynamicsConfig {
    pub minimum_samples: usize,
}

impl Default for DynamicsConfig {
    fn default() -> Self {
        Self { minimum_samples: 2 }
    }
}

impl DynamicsConfig {
    pub fn validate(self) -> Result<(), DynamicsError> {
        if self.minimum_samples < 2 {
            Err(DynamicsError::InvalidConfig)
        } else {
            Ok(())
        }
    }
}
