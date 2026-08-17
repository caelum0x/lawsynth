//! The job execution core.
//!
//! This is the single place that maps a typed [`Job`] onto the underlying
//! discovery and simulation engines and honours cooperative cancellation. It is
//! deliberately free of admission, checkpointing, and transport concerns so the
//! same logic can be exercised directly in tests and reused by [`crate::Worker`].

use lawsynth_runner::CancellationToken;

use crate::{Job, JobOutput, WorkerError};

/// Runs one typed job to completion, mapping engine errors into [`WorkerError`].
///
/// Cancellation is checked once after the engine returns: the engines are
/// synchronous, so a token flipped during execution is observed on the way out
/// and turns any result into [`WorkerError::Cancelled`].
pub(crate) fn run(job: &Job, cancellation: &CancellationToken) -> Result<JobOutput, WorkerError> {
    let result = match job {
        Job::Discover { dataset, config } => lawsynth_discovery::discover(dataset, config)
            .map(JobOutput::Discovery)
            .map_err(WorkerError::from),
        Job::Simulate { world, config, request } => lawsynth_sim::simulate(world, *config, request)
            .map(JobOutput::Simulation)
            .map_err(WorkerError::from),
    };
    match cancellation.reason() {
        Some(reason) => Err(WorkerError::Cancelled(reason)),
        None => result,
    }
}

/// A short, human-readable summary of a completed job's output, recorded in the
/// terminal checkpoint's detail field.
pub(crate) fn output_summary(output: &JobOutput) -> String {
    match output {
        JobOutput::Discovery(result) => {
            format!("discovery completed with {} Pareto candidate(s)", result.candidates.len())
        }
        JobOutput::Simulation(trajectory) => {
            format!("simulation completed with {} sample(s)", trajectory.samples())
        }
    }
}
