use crate::PluginError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PluginState {
    Discovered,
    Validated,
    Starting,
    Running,
    Draining,
    Stopped,
    Failed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleEvent {
    Validate,
    Start,
    Ready,
    Drain,
    Stop,
    Fail,
}

impl PluginState {
    pub fn transition(self, event: LifecycleEvent) -> Result<Self, PluginError> {
        use LifecycleEvent::*;
        use PluginState::*;
        let next = match (self, event) {
            (Discovered, Validate) => Validated,
            (Validated, Start) => Starting,
            (Starting, Ready) => Running,
            (Running, Drain) => Draining,
            (Draining, Stop) | (Running, Stop) | (Starting, Stop) => Stopped,
            (Discovered | Validated | Starting | Running | Draining, Fail) => Failed,
            _ => {
                return Err(PluginError::InvalidState {
                    from: format!("{self:?}"),
                    event: format!("{event:?}"),
                });
            }
        };
        Ok(next)
    }
    pub const fn accepts_requests(self) -> bool {
        matches!(self, Self::Running)
    }
}
