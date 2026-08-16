use crate::HostError;
use lawsynth_plugin_api::{Capability, CapabilitySet, PluginKind, PluginManifest};

/// Permissions granted by an administrator, independently from manifest claims.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PermissionSet(pub CapabilitySet);
impl PermissionSet {
    pub fn allows(&self, capability: Capability) -> bool {
        self.0.contains(capability)
    }
    pub fn iter(&self) -> impl Iterator<Item = Capability> + '_ {
        self.0.iter()
    }
}

/// Default-deny policy. Native code needs a separate deliberate opt-in.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PermissionPolicy {
    pub allowed: PermissionSet,
    pub allow_trusted_native: bool,
}
impl PermissionPolicy {
    pub fn grant(&self, manifest: &PluginManifest) -> Result<PermissionSet, HostError> {
        if manifest.kind == PluginKind::TrustedNative && !self.allow_trusted_native {
            return Err(HostError::PermissionDenied(
                "trusted native plugins require explicit host opt-in".into(),
            ));
        }
        if !manifest.capabilities.is_subset_of(&self.allowed.0) {
            let denied = manifest
                .capabilities
                .iter()
                .find(|c| !self.allowed.0.contains(*c))
                .expect("subset was false");
            return Err(HostError::PermissionDenied(denied.as_str().into()));
        }
        Ok(PermissionSet(manifest.capabilities.clone()))
    }
}
