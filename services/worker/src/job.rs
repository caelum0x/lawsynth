use lawsynth_data::Dataset;
use lawsynth_discovery::{DiscoveryConfig, DiscoveryResult};
use lawsynth_runner::{ResourceRequest, WorkEnvelope};
use lawsynth_sim::{SimulationConfig, SimulationRequest, Trajectory};
use lawsynth_world::World;

use crate::WorkerError;

/// Typed work accepted by the local worker. The payload is in-memory on purpose:
/// this crate does not pretend to provide a queue codec or a network API.
#[derive(Clone, Debug, PartialEq)]
pub enum Job {
    Discover { dataset: Dataset, config: DiscoveryConfig },
    Simulate { world: World, config: SimulationConfig, request: SimulationRequest },
}

impl Job {
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Discover { .. } => "discover",
            Self::Simulate { .. } => "simulate",
        }
    }

    fn validate(&self) -> Result<(), WorkerError> {
        match self {
            Self::Discover { dataset, config } => {
                if dataset.time().len() < 3 {
                    return Err(WorkerError::InvalidJob(
                        "discovery requires at least three samples".into(),
                    ));
                }
                if config.state.is_empty() {
                    return Err(WorkerError::InvalidJob(
                        "discovery requires at least one state".into(),
                    ));
                }
            }
            Self::Simulate { config, .. } => {
                SimulationConfig::new(config.start, config.end, config.step)
                    .map_err(WorkerError::from)?;
            }
        }
        Ok(())
    }
}

/// Work envelope validated before resource admission. `attempt` begins at one and
/// input bytes are intentionally absent because the executable job is typed.
#[derive(Clone, Debug, PartialEq)]
pub struct JobEnvelope {
    pub work: WorkEnvelope,
    pub job: Job,
}

impl JobEnvelope {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        attempt: u32,
        submitted_at_ms: u64,
        deadline_at_ms: u64,
        resources: ResourceRequest,
        job: Job,
    ) -> Result<Self, WorkerError> {
        job.validate()?;
        let work = WorkEnvelope::new(
            id,
            job.kind(),
            attempt,
            submitted_at_ms,
            deadline_at_ms,
            resources,
            Vec::new(),
        )?;
        Ok(Self { work, job })
    }
}

/// Output remains typed for the in-process caller, avoiding lossy ad-hoc result codecs.
#[derive(Clone, Debug, PartialEq)]
pub enum JobOutput {
    Discovery(DiscoveryResult),
    Simulation(Trajectory),
}

/// Queueing and network listeners are intentionally not part of this worker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportSurface {
    LocalDirect,
    HttpStatus,
    QueueNotImplemented,
    NetworkNotImplemented,
}

impl TransportSurface {
    /// Whether the surface can admit executable jobs. Only in-process typed
    /// execution can: the HTTP surface is status-only and the queue/network
    /// surfaces are unimplemented.
    pub const fn is_available(self) -> bool {
        matches!(self, Self::LocalDirect)
    }
    pub const fn reason(self) -> &'static str {
        match self {
            Self::LocalDirect => "in-process typed execution",
            Self::HttpStatus => {
                "serves health, admission, and lifecycle status only; it does not accept executable jobs"
            }
            Self::QueueNotImplemented => "no queue client or message codec is linked",
            Self::NetworkNotImplemented => "no HTTP, RPC, or authentication transport is linked",
        }
    }
}
