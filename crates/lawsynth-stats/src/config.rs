use crate::StatsError;

/// Configuration for deterministic histogram information estimates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HistogramConfig {
    pub bins: usize,
}

impl Default for HistogramConfig {
    fn default() -> Self {
        Self { bins: 16 }
    }
}

impl HistogramConfig {
    pub fn validate(self) -> Result<Self, StatsError> {
        (self.bins > 0)
            .then_some(self)
            .ok_or(StatsError::InvalidHistogramConfig)
    }
}
