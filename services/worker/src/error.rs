use std::fmt;

#[derive(Debug)]
pub enum WorkerError {
    InvalidConfig(String),
    InvalidJob(String),
    DeadlineExceeded { job_id: String, deadline_at_ms: u64 },
    DuplicateJob(String),
    Cancelled(String),
    Runner(lawsynth_runner::RunnerError),
    Discovery(lawsynth_discovery::DiscoveryError),
    Simulation(lawsynth_sim::SimulationError),
    Store(lawsynth_store::StoreError),
    CorruptCheckpoint(String),
    UnsupportedTransport(&'static str),
}

impl fmt::Display for WorkerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(reason) => {
                write!(formatter, "invalid worker configuration: {reason}")
            }
            Self::InvalidJob(reason) => write!(formatter, "invalid worker job: {reason}"),
            Self::DeadlineExceeded { job_id, deadline_at_ms } => {
                write!(formatter, "job '{job_id}' exceeded deadline {deadline_at_ms}")
            }
            Self::DuplicateJob(id) => {
                write!(formatter, "job '{id}' already has a durable lifecycle record")
            }
            Self::Cancelled(reason) => write!(formatter, "job cancelled: {reason}"),
            Self::Runner(error) => error.fmt(formatter),
            Self::Discovery(error) => error.fmt(formatter),
            Self::Simulation(error) => error.fmt(formatter),
            Self::Store(error) => error.fmt(formatter),
            Self::CorruptCheckpoint(reason) => {
                write!(formatter, "corrupt worker checkpoint: {reason}")
            }
            Self::UnsupportedTransport(reason) => {
                write!(formatter, "unsupported worker transport: {reason}")
            }
        }
    }
}

impl std::error::Error for WorkerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runner(error) => Some(error),
            Self::Discovery(error) => Some(error),
            Self::Simulation(error) => Some(error),
            Self::Store(error) => Some(error),
            _ => None,
        }
    }
}

impl From<lawsynth_runner::RunnerError> for WorkerError {
    fn from(value: lawsynth_runner::RunnerError) -> Self {
        Self::Runner(value)
    }
}
impl From<lawsynth_discovery::DiscoveryError> for WorkerError {
    fn from(value: lawsynth_discovery::DiscoveryError) -> Self {
        Self::Discovery(value)
    }
}
impl From<lawsynth_sim::SimulationError> for WorkerError {
    fn from(value: lawsynth_sim::SimulationError) -> Self {
        Self::Simulation(value)
    }
}
impl From<lawsynth_store::StoreError> for WorkerError {
    fn from(value: lawsynth_store::StoreError) -> Self {
        Self::Store(value)
    }
}
