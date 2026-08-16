use lawsynth_plugin_api::PluginError;
use std::fmt;

#[derive(Debug)]
pub enum HostError {
    Api(PluginError),
    Io(std::io::Error),
    Disabled,
    PermissionDenied(String),
    AlreadyRegistered(String),
    NotRegistered(String),
    Process(String),
    Resource(String),
}
impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Api(e) => e.fmt(f),
            Self::Io(e) => e.fmt(f),
            Self::Disabled => write!(f, "plugins are disabled by host policy"),
            Self::PermissionDenied(v) => write!(f, "plugin permission denied: {v}"),
            Self::AlreadyRegistered(v) => write!(f, "plugin already registered: {v}"),
            Self::NotRegistered(v) => write!(f, "plugin is not registered: {v}"),
            Self::Process(v) => write!(f, "plugin process error: {v}"),
            Self::Resource(v) => write!(f, "plugin resource error: {v}"),
        }
    }
}
impl std::error::Error for HostError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Api(e) => Some(e),
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}
impl From<PluginError> for HostError {
    fn from(value: PluginError) -> Self {
        Self::Api(value)
    }
}
impl From<std::io::Error> for HostError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
