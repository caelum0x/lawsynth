use std::fmt;

/// A protocol or extension contract violation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PluginError {
    InvalidManifest(String),
    InvalidCapability(String),
    InvalidData(String),
    InvalidLimits(String),
    InvalidState { from: String, event: String },
    Protocol(String),
    ResourceLimit(String),
    Unsupported(String),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidManifest(v) => write!(f, "invalid plugin manifest: {v}"),
            Self::InvalidCapability(v) => write!(f, "invalid capability: {v}"),
            Self::InvalidData(v) => write!(f, "invalid plugin data: {v}"),
            Self::InvalidLimits(v) => write!(f, "invalid resource limits: {v}"),
            Self::InvalidState { from, event } => {
                write!(f, "cannot apply {event} while plugin is {from}")
            }
            Self::Protocol(v) => write!(f, "plugin protocol error: {v}"),
            Self::ResourceLimit(v) => write!(f, "plugin resource limit exceeded: {v}"),
            Self::Unsupported(v) => write!(f, "unsupported plugin feature: {v}"),
        }
    }
}

impl std::error::Error for PluginError {}
