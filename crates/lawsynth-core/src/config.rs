use crate::{CURRENT_ENGINE_VERSION, EngineVersion, ResourceLimits, Seed};

/// Shared deterministic execution settings for public engine entry points.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EngineConfig {
    pub version: EngineVersion,
    pub seed: Seed,
    pub resource_limits: ResourceLimits,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            version: CURRENT_ENGINE_VERSION,
            seed: Seed::default(),
            resource_limits: ResourceLimits::default(),
        }
    }
}
