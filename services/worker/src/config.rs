use lawsynth_runner::ResourceRequest;

use crate::WorkerError;

/// Capacity and persistence bounds for one in-process worker instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerConfig {
    pub capacity: ResourceRequest,
    pub maximum_checkpoint_bytes: usize,
}

impl WorkerConfig {
    pub fn new(
        capacity: ResourceRequest,
        maximum_checkpoint_bytes: usize,
    ) -> Result<Self, WorkerError> {
        if maximum_checkpoint_bytes < 128 {
            return Err(WorkerError::InvalidConfig(
                "maximum_checkpoint_bytes must allow a complete lifecycle record".into(),
            ));
        }
        Ok(Self { capacity, maximum_checkpoint_bytes })
    }
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            capacity: ResourceRequest::new(4_000, 4 * 1024 * 1024 * 1024, 8 * 1024 * 1024 * 1024)
                .expect("built-in worker capacity is valid"),
            maximum_checkpoint_bytes: 16 * 1024,
        }
    }
}
