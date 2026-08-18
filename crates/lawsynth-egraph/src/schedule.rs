use crate::{RewriteConfig, RewriteError};

/// A validated bounded schedule for saturation passes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RewriteSchedule {
    pub passes: usize,
}
impl RewriteSchedule {
    pub fn from_config(config: &RewriteConfig) -> Result<Self, RewriteError> {
        if config.max_passes == 0 {
            Err(RewriteError::InvalidConfig)
        } else {
            Ok(Self { passes: config.max_passes })
        }
    }
}
