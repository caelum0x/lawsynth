use std::fmt;

#[derive(Debug)]
pub enum SchedulerError {
    InvalidConfig(String),
    InvalidWorker(String),
    QueueFull { limit: usize },
    DuplicateJob(String),
    UnknownJob(String),
    UnknownWorker(String),
    StaleLease { job_id: String },
    LeaseExpired { job_id: String },
    InvalidTransition { job_id: String, state: &'static str },
    CheckpointTooLarge { actual: usize, limit: usize },
    CorruptCheckpoint(String),
    UnsupportedTransport(&'static str),
    Store(lawsynth_store::StoreError),
}

impl fmt::Display for SchedulerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(reason) => {
                write!(formatter, "invalid scheduler configuration: {reason}")
            }
            Self::InvalidWorker(reason) => write!(formatter, "invalid worker pool: {reason}"),
            Self::QueueFull { limit } => {
                write!(formatter, "scheduler queue reached its {limit} job limit")
            }
            Self::DuplicateJob(id) => {
                write!(formatter, "job '{id}' is already known to the scheduler")
            }
            Self::UnknownJob(id) => write!(formatter, "job '{id}' is not known to the scheduler"),
            Self::UnknownWorker(id) => write!(formatter, "worker pool '{id}' is not registered"),
            Self::StaleLease { job_id } => {
                write!(formatter, "lease for job '{job_id}' is stale or fenced")
            }
            Self::LeaseExpired { job_id } => {
                write!(formatter, "lease for job '{job_id}' has expired")
            }
            Self::InvalidTransition { job_id, state } => {
                write!(formatter, "job '{job_id}' cannot transition from {state}")
            }
            Self::CheckpointTooLarge { actual, limit } => {
                write!(formatter, "scheduler checkpoint has {actual} bytes; limit is {limit}")
            }
            Self::CorruptCheckpoint(reason) => {
                write!(formatter, "corrupt scheduler checkpoint: {reason}")
            }
            Self::UnsupportedTransport(reason) => {
                write!(formatter, "unsupported scheduler transport: {reason}")
            }
            Self::Store(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for SchedulerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Store(error) => Some(error),
            _ => None,
        }
    }
}

impl From<lawsynth_store::StoreError> for SchedulerError {
    fn from(value: lawsynth_store::StoreError) -> Self {
        Self::Store(value)
    }
}
