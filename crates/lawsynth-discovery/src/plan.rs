use crate::{DiscoveryConfig, DiscoveryStage};
use lawsynth_core::Identifier;

/// Immutable execution plan derived from user configuration before discovery starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveryPlan {
    pub states: Vec<Identifier>,
    pub stages: Vec<DiscoveryStage>,
}
impl DiscoveryPlan {
    pub fn from_config(config: &DiscoveryConfig) -> Self {
        Self {
            states: config.state.clone(),
            stages: DiscoveryStage::all().into(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
}
