use std::time::Duration;

use crate::RunnerError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunnerConfig {
    pub heartbeat_interval: Duration,
    pub stale_after: Duration,
    pub maximum_checkpoint_bytes: usize,
    pub maximum_attempts: u32,
}

impl RunnerConfig {
    pub fn new(
        heartbeat_interval: Duration,
        stale_after: Duration,
        maximum_checkpoint_bytes: usize,
        maximum_attempts: u32,
    ) -> Result<Self, RunnerError> {
        if heartbeat_interval.is_zero() {
            return Err(RunnerError::InvalidConfig(
                "heartbeat_interval must be positive",
            ));
        }
        if stale_after <= heartbeat_interval {
            return Err(RunnerError::InvalidConfig(
                "stale_after must exceed heartbeat_interval",
            ));
        }
        if maximum_checkpoint_bytes == 0 {
            return Err(RunnerError::InvalidConfig(
                "maximum_checkpoint_bytes must be positive",
            ));
        }
        if maximum_attempts == 0 {
            return Err(RunnerError::InvalidConfig(
                "maximum_attempts must be positive",
            ));
        }
        Ok(Self {
            heartbeat_interval,
            stale_after,
            maximum_checkpoint_bytes,
            maximum_attempts,
        })
    }
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(5),
            stale_after: Duration::from_secs(30),
            maximum_checkpoint_bytes: 16 << 20,
            maximum_attempts: 3,
        }
    }
}
