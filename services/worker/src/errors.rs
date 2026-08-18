//! The worker's consolidated error taxonomy.
//!
//! This module is the single source of truth for [`WorkerError`]. Every worker
//! module -- execution, admission, sandboxing, artifact handoff, the plugin
//! seam, recovery, and the HTTP status transport -- reports failures through
//! this one enum, so there is no competing error type to keep in sync. The
//! transport-status mapping lives in [`crate::http_error`], which matches on
//! these variants exhaustively.

use std::fmt;

/// Every failure the worker can surface, from a single authoritative definition.
#[derive(Debug)]
pub enum WorkerError {
    /// The worker was configured with values that cannot admit a valid job.
    InvalidConfig(String),
    /// A submitted job failed validation before admission.
    InvalidJob(String),
    /// The job's deadline had already elapsed at admission time.
    DeadlineExceeded { job_id: String, deadline_at_ms: u64 },
    /// A durable lifecycle record already exists for this job id.
    DuplicateJob(String),
    /// The job was cooperatively cancelled.
    Cancelled(String),
    /// A configured sandbox/admission bound was exceeded by the job.
    LimitExceeded(String),
    /// The plugin execution seam could not satisfy the request (e.g. no host
    /// is linked). This is an honest "unsupported", never a faked success.
    Plugin(String),
    /// The produced-artifact handoff failed integrity or upload verification.
    Artifact(String),
    /// A failure propagated from the runtime substrate.
    Runner(lawsynth_runner::RunnerError),
    /// A failure propagated from the discovery engine.
    Discovery(lawsynth_discovery::DiscoveryError),
    /// A failure propagated from the simulation engine.
    Simulation(lawsynth_sim::SimulationError),
    /// A failure propagated from the stability-analysis engine.
    Stability(lawsynth_stability::StabilityError),
    /// A failure propagated from the object store.
    Store(lawsynth_store::StoreError),
    /// A persisted checkpoint could not be parsed and must not be trusted.
    CorruptCheckpoint(String),
    /// A transport surface that the worker deliberately does not implement.
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
            Self::LimitExceeded(reason) => write!(formatter, "resource limit exceeded: {reason}"),
            Self::Plugin(reason) => write!(formatter, "plugin dispatch unavailable: {reason}"),
            Self::Artifact(reason) => write!(formatter, "artifact handoff failed: {reason}"),
            Self::Runner(error) => error.fmt(formatter),
            Self::Discovery(error) => error.fmt(formatter),
            Self::Simulation(error) => error.fmt(formatter),
            Self::Stability(error) => error.fmt(formatter),
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
            Self::Stability(error) => Some(error),
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
impl From<lawsynth_stability::StabilityError> for WorkerError {
    fn from(value: lawsynth_stability::StabilityError) -> Self {
        Self::Stability(value)
    }
}
impl From<lawsynth_store::StoreError> for WorkerError {
    fn from(value: lawsynth_store::StoreError) -> Self {
        Self::Store(value)
    }
}
