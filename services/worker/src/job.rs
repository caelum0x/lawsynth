use lawsynth_data::Dataset;
use lawsynth_discovery::{DiscoveryConfig, DiscoveryResult};
use lawsynth_runner::{ResourceRequest, WorkEnvelope};
use lawsynth_sim::{SimulationConfig, SimulationRequest, Trajectory};
use lawsynth_stability::{StabilityConfig, StabilityReport};
use lawsynth_world::World;

use crate::WorkerError;

/// Typed work accepted by the local worker. The payload is in-memory on purpose:
/// this crate does not pretend to provide a queue codec or a network API.
///
/// The variants differ in size, but boxing the larger one would change the
/// public shape that in-process callers (the scheduler and executor) destructure
/// by variant, so the difference is accepted rather than hidden behind
/// indirection -- mirroring the same decision documented on [`JobOutput`].
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum Job {
    Discover {
        dataset: Dataset,
        config: DiscoveryConfig,
    },
    Simulate {
        world: World,
        config: SimulationConfig,
        request: SimulationRequest,
    },
    /// Fixed-point and linear-stability analysis of a world's autonomous vector
    /// field over a caller-provided search box. The `config` carries both the
    /// per-state search box and every deterministic Newton/classification knob.
    AnalyzeStability {
        world: World,
        config: StabilityConfig,
    },
}

impl Job {
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Discover { .. } => "discover",
            Self::Simulate { .. } => "simulate",
            Self::AnalyzeStability { .. } => "analyze-stability",
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
            Self::AnalyzeStability { world, config } => {
                let states = world.state_ids().count();
                if states == 0 {
                    return Err(WorkerError::InvalidJob(
                        "stability analysis requires at least one state".into(),
                    ));
                }
                if world.laws().is_empty() {
                    return Err(WorkerError::InvalidJob(
                        "stability analysis requires at least one law".into(),
                    ));
                }
                let intervals = config.search_box().len();
                if intervals != states {
                    return Err(WorkerError::InvalidJob(format!(
                        "stability search box has {intervals} interval(s) but the world has \
{states} state(s)"
                    )));
                }
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
///
/// The variants differ in size, but boxing the larger one would change the
/// public shape that in-process callers (including the scheduler) destructure,
/// so the difference is accepted rather than hidden behind indirection.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum JobOutput {
    Discovery(DiscoveryResult),
    Simulation(Trajectory),
    Stability(StabilityReport),
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

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;
    use lawsynth_expr::Expr;
    use lawsynth_stability::StabilityConfig;
    use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

    use super::Job;
    use crate::WorkerError;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    /// A one-state world `x' = x`, enough to exercise stability validation.
    fn one_state_world() -> World {
        World::new(
            [Variable::new(id("x"), VariableRole::State)],
            [],
            [ContinuousLaw::new(id("x"), Expr::symbol(id("x")))],
        )
        .unwrap()
    }

    #[test]
    fn kind_of_stability_job_is_analyze_stability() {
        let job = Job::AnalyzeStability {
            world: one_state_world(),
            config: StabilityConfig::new(vec![(-1.0, 1.0)]),
        };
        assert_eq!(job.kind(), "analyze-stability");
    }

    #[test]
    fn validate_accepts_matching_box_dimension() {
        let job = Job::AnalyzeStability {
            world: one_state_world(),
            config: StabilityConfig::new(vec![(-1.0, 1.0)]),
        };
        assert!(job.validate().is_ok());
    }

    #[test]
    fn validate_rejects_box_and_state_dimension_mismatch() {
        // One state, but a two-interval search box.
        let job = Job::AnalyzeStability {
            world: one_state_world(),
            config: StabilityConfig::new(vec![(-1.0, 1.0), (-1.0, 1.0)]),
        };
        let error = job.validate().unwrap_err();
        assert!(matches!(error, WorkerError::InvalidJob(_)));
        let message = error.to_string();
        assert!(message.contains("interval"));
        assert!(message.contains("state"));
    }
}
