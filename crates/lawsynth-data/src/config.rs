use lawsynth_core::{ResourceLimitError, ResourceLimits};

use crate::Dataset;

/// Opt-in bounds applied before expensive dataset-consuming operations.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DatasetConfig {
    pub resource_limits: ResourceLimits,
}

impl DatasetConfig {
    pub fn validate(&self, dataset: &Dataset) -> Result<(), ResourceLimitError> {
        self.resource_limits
            .validate_dataset(dataset.time().len(), dataset.columns().len())
    }
}
