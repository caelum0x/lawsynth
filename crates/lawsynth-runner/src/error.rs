use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RunnerError {
    InvalidConfig(&'static str),
    InvalidEnvelope(&'static str),
    CapacityExceeded { requested: u64, available: u64 },
    Cancelled { reason: String },
    CheckpointRejected(&'static str),
    ProcessFailed(String),
}

impl fmt::Display for RunnerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(reason) => write!(formatter, "invalid runner config: {reason}"),
            Self::InvalidEnvelope(reason) => write!(formatter, "invalid work envelope: {reason}"),
            Self::CapacityExceeded {
                requested,
                available,
            } => write!(
                formatter,
                "requested {requested} units; only {available} available"
            ),
            Self::Cancelled { reason } => write!(formatter, "work cancelled: {reason}"),
            Self::CheckpointRejected(reason) => write!(formatter, "checkpoint rejected: {reason}"),
            Self::ProcessFailed(reason) => write!(formatter, "process failed: {reason}"),
        }
    }
}

impl std::error::Error for RunnerError {}
