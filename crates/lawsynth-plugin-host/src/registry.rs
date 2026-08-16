use crate::{HostConfig, HostError, PermissionSet};
use lawsynth_plugin_api::PluginManifest;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct RegisteredPlugin {
    pub manifest: PluginManifest,
    pub permissions: PermissionSet,
}
#[derive(Clone, Debug, Default)]
pub struct PluginRegistry {
    entries: BTreeMap<String, RegisteredPlugin>,
}
impl PluginRegistry {
    pub fn register(
        &mut self,
        config: &HostConfig,
        manifest: PluginManifest,
    ) -> Result<(), HostError> {
        if !config.enabled {
            return Err(HostError::Disabled);
        }
        manifest.validate()?;
        if !config.maximum_limits.permits(manifest.limits) {
            return Err(HostError::Resource(
                "manifest requests more than host maximum".into(),
            ));
        }
        let permissions = config.policy.grant(&manifest)?;
        if self.entries.contains_key(&manifest.id) {
            return Err(HostError::AlreadyRegistered(manifest.id));
        }
        self.entries.insert(
            manifest.id.clone(),
            RegisteredPlugin {
                manifest,
                permissions,
            },
        );
        Ok(())
    }
    pub fn get(&self, id: &str) -> Result<&RegisteredPlugin, HostError> {
        self.entries
            .get(id)
            .ok_or_else(|| HostError::NotRegistered(id.into()))
    }
    pub fn remove(&mut self, id: &str) -> Result<RegisteredPlugin, HostError> {
        self.entries
            .remove(id)
            .ok_or_else(|| HostError::NotRegistered(id.into()))
    }
    pub fn iter(&self) -> impl Iterator<Item = (&str, &RegisteredPlugin)> {
        self.entries
            .iter()
            .map(|(id, plugin)| (id.as_str(), plugin))
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
