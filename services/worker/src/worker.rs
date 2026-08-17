use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use lawsynth_runner::{CancellationToken, ResourceLimiter, ResourceRequest};
use lawsynth_store::ObjectStore;

use crate::{
    CheckpointState, Job, JobCheckpoint, JobEnvelope, JobOutput, WorkerConfig, WorkerError,
    checkpoint,
};

/// Synchronous local execution engine. The object store is the authority for
/// lifecycle checkpoints; the resource limiter makes concurrent callers share
/// an explicit capacity budget.
pub struct Worker<S> {
    config: WorkerConfig,
    store: S,
    limiter: Mutex<ResourceLimiter>,
}

impl<S: ObjectStore> Worker<S> {
    pub fn new(config: WorkerConfig, store: S) -> Result<Self, WorkerError> {
        Ok(Self { limiter: Mutex::new(ResourceLimiter::new(config.capacity)), config, store })
    }
    pub fn config(&self) -> &WorkerConfig {
        &self.config
    }
    pub fn checkpoint(&self, job_id: &str) -> Result<Option<JobCheckpoint>, WorkerError> {
        checkpoint::load(&self.store, job_id)
    }

    /// Snapshots the current admission budget for the read-only status surface.
    /// Locking the limiter briefly yields a consistent capacity/reserved/available
    /// triple without affecting in-flight admission.
    pub fn admission(&self) -> AdmissionSnapshot {
        let limiter = self.limiter.lock().expect("worker resource limiter mutex poisoned");
        AdmissionSnapshot {
            capacity: limiter.capacity(),
            reserved: limiter.reserved(),
            available: limiter.available(),
        }
    }

    /// Returns the ids of every job for which a durable checkpoint exists,
    /// read from the object store rather than any in-memory index.
    pub fn known_checkpoints(&self) -> Result<Vec<String>, WorkerError> {
        checkpoint::list(&self.store)
    }
    pub fn execute(
        &self,
        envelope: &JobEnvelope,
        cancellation: &CancellationToken,
    ) -> Result<JobOutput, WorkerError> {
        self.execute_at(envelope, cancellation, now_ms())
    }

    /// Runs one typed job at a caller-supplied clock instant. Supplying time makes
    /// deadlines and checkpoint records reproducible in integration tests.
    pub fn execute_at(
        &self,
        envelope: &JobEnvelope,
        cancellation: &CancellationToken,
        now_ms: u64,
    ) -> Result<JobOutput, WorkerError> {
        if envelope.work.is_expired(now_ms) {
            return self.reject(
                envelope,
                now_ms,
                WorkerError::DeadlineExceeded {
                    job_id: envelope.work.id.clone(),
                    deadline_at_ms: envelope.work.deadline_at_ms,
                },
            );
        }
        if let Some(existing) = self.checkpoint(&envelope.work.id)? {
            return Err(WorkerError::DuplicateJob(format!(
                "{} ({})",
                existing.job_id,
                existing.state.as_str()
            )));
        }
        if let Some(reason) = cancellation.reason() {
            return self.finish_without_admission(
                envelope,
                now_ms,
                CheckpointState::Cancelled,
                reason.clone(),
                WorkerError::Cancelled(reason),
            );
        }

        let mut limiter = self.limiter.lock().expect("worker resource limiter mutex poisoned");
        if let Err(error) = limiter.reserve(envelope.work.resources) {
            return self.reject(envelope, now_ms, error.into());
        }
        let result = self.run_reserved(envelope, cancellation, now_ms);
        let release = limiter.release(envelope.work.resources).map_err(WorkerError::from);
        drop(limiter);
        release?;
        result
    }

    fn run_reserved(
        &self,
        envelope: &JobEnvelope,
        cancellation: &CancellationToken,
        now_ms: u64,
    ) -> Result<JobOutput, WorkerError> {
        self.write_next(envelope, now_ms, CheckpointState::Running, "resources admitted")?;
        let result = match &envelope.job {
            Job::Discover { dataset, config } => lawsynth_discovery::discover(dataset, config)
                .map(JobOutput::Discovery)
                .map_err(WorkerError::from),
            Job::Simulate { world, config, request } => {
                lawsynth_sim::simulate(world, *config, request)
                    .map(JobOutput::Simulation)
                    .map_err(WorkerError::from)
            }
        };
        let result = match cancellation.reason() {
            Some(reason) => Err(WorkerError::Cancelled(reason)),
            None => result,
        };
        let (state, detail) = match &result {
            Ok(output) => (CheckpointState::Completed, output_summary(output)),
            Err(WorkerError::Cancelled(reason)) => (CheckpointState::Cancelled, reason.clone()),
            Err(error) => (CheckpointState::Failed, error.to_string()),
        };
        self.write_next(envelope, now_ms, state, &detail)?;
        result
    }

    fn reject<T>(
        &self,
        envelope: &JobEnvelope,
        now_ms: u64,
        error: WorkerError,
    ) -> Result<T, WorkerError> {
        self.finish_without_admission(
            envelope,
            now_ms,
            CheckpointState::Rejected,
            error.to_string(),
            error,
        )
    }
    fn finish_without_admission<T>(
        &self,
        envelope: &JobEnvelope,
        now_ms: u64,
        state: CheckpointState,
        detail: String,
        error: WorkerError,
    ) -> Result<T, WorkerError> {
        self.write_next(envelope, now_ms, state, &detail)?;
        Err(error)
    }
    fn write_next(
        &self,
        envelope: &JobEnvelope,
        now_ms: u64,
        state: CheckpointState,
        detail: &str,
    ) -> Result<(), WorkerError> {
        let previous = self.checkpoint(&envelope.work.id)?;
        let sequence = match previous {
            Some(record) => record.sequence.checked_add(1).ok_or_else(|| {
                WorkerError::CorruptCheckpoint("checkpoint sequence overflow".into())
            })?,
            None => 1,
        };
        checkpoint::save(
            &self.store,
            &JobCheckpoint {
                job_id: envelope.work.id.clone(),
                sequence,
                recorded_at_ms: now_ms,
                state,
                detail: detail.to_owned(),
            },
            self.config.maximum_checkpoint_bytes,
        )
    }
}

/// A consistent view of the worker's admission budget for status reporting.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmissionSnapshot {
    pub capacity: ResourceRequest,
    pub reserved: ResourceRequest,
    pub available: ResourceRequest,
}

fn output_summary(output: &JobOutput) -> String {
    match output {
        JobOutput::Discovery(result) => {
            format!("discovery completed with {} Pareto candidate(s)", result.candidates.len())
        }
        JobOutput::Simulation(trajectory) => {
            format!("simulation completed with {} sample(s)", trajectory.samples())
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}
