use crate::{HostError, ResourceMeter};
use lawsynth_plugin_api::{LifecycleEvent, PluginManifest, PluginState, ResourceLimits};

/// Host-owned lifecycle state. Calls are rejected unless the plugin is running.
#[derive(Debug)]
pub struct HostedPlugin {
    pub manifest: PluginManifest,
    state: PluginState,
    meter: ResourceMeter,
}
impl HostedPlugin {
    pub fn new(manifest: PluginManifest, limits: ResourceLimits) -> Result<Self, HostError> {
        manifest.validate()?;
        Ok(Self {
            manifest,
            state: PluginState::Discovered,
            meter: ResourceMeter::new(limits)?,
        })
    }
    pub const fn state(&self) -> PluginState {
        self.state
    }
    pub fn apply(&mut self, event: LifecycleEvent) -> Result<PluginState, HostError> {
        self.state = self.state.transition(event)?;
        Ok(self.state)
    }
    pub fn begin_request(&mut self) -> Result<(), HostError> {
        if !self.state.accepts_requests() {
            return Err(HostError::Process("plugin is not running".into()));
        }
        self.meter.begin_request()
    }
    pub fn record_output(&mut self, bytes: usize) -> Result<(), HostError> {
        self.meter.record_output(bytes)
    }
}
