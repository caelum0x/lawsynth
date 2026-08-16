use crate::permissions::PermissionPolicy;
use lawsynth_plugin_api::ResourceLimits;

/// Global host policy. Plugins are off by default for server-safe embedding.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HostConfig {
    pub enabled: bool,
    pub policy: PermissionPolicy,
    pub maximum_limits: ResourceLimits,
}
