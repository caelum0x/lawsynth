use std::time::Duration;

use crate::SchedulerError;

/// Bounds for one synchronous scheduler instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulerConfig {
    pub maximum_queued_jobs: usize,
    pub maximum_attempts: u32,
    pub lease_duration: Duration,
    pub maximum_checkpoint_bytes: usize,
}

impl SchedulerConfig {
    pub fn new(
        maximum_queued_jobs: usize,
        maximum_attempts: u32,
        lease_duration: Duration,
        maximum_checkpoint_bytes: usize,
    ) -> Result<Self, SchedulerError> {
        if maximum_queued_jobs == 0 {
            return Err(SchedulerError::InvalidConfig(
                "maximum_queued_jobs must be positive".into(),
            ));
        }
        if maximum_attempts == 0 {
            return Err(SchedulerError::InvalidConfig("maximum_attempts must be positive".into()));
        }
        if lease_duration.is_zero() {
            return Err(SchedulerError::InvalidConfig("lease_duration must be positive".into()));
        }
        // The durable format hex-encodes a bounded (2 KiB) failure reason and
        // can include maximum-length job and worker IDs. Keep every valid
        // lifecycle transition persistable rather than failing after mutation.
        if maximum_checkpoint_bytes < 5 * 1024 {
            return Err(SchedulerError::InvalidConfig(
                "maximum_checkpoint_bytes must be at least 5 KiB".into(),
            ));
        }
        Ok(Self { maximum_queued_jobs, maximum_attempts, lease_duration, maximum_checkpoint_bytes })
    }
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            maximum_queued_jobs: 10_000,
            maximum_attempts: 3,
            lease_duration: Duration::from_secs(30),
            maximum_checkpoint_bytes: 16 * 1024,
        }
    }
}
