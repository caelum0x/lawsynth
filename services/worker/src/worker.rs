use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use lawsynth_runner::{CancellationToken, ResourceLimiter};
use lawsynth_store::ObjectStore;

use crate::{
    AdmissionSnapshot, ArtifactReceipt, CheckpointState, CleanupReport, EventLog, HealthSnapshot,
    JobCheckpoint, JobEnvelope, JobEvent, JobOutput, Limits, RecoveryPlan, Sandbox, Telemetry,
    TelemetrySnapshot, WorkerConfig, WorkerError, artifacts, checkpoint, cleanup, execute,
    recovery, resources,
};

/// Synchronous local execution engine. The object store is the authority for
/// lifecycle checkpoints; the resource limiter makes concurrent callers share
/// an explicit capacity budget.
///
/// The worker composes its behaviour from focused modules: [`execute`] runs the
/// typed job, [`resources`] accounts admission, [`crate::sandbox`] enforces the
/// deadline and any configured per-job bounds, and every lifecycle transition
/// updates [`Telemetry`] counters and appends to the [`EventLog`].
pub struct Worker<S> {
    config: WorkerConfig,
    store: S,
    limiter: Mutex<ResourceLimiter>,
    sandbox: Sandbox,
    telemetry: Telemetry,
    events: Mutex<EventLog>,
}

impl<S: ObjectStore> Worker<S> {
    pub fn new(config: WorkerConfig, store: S) -> Result<Self, WorkerError> {
        let sandbox = Sandbox::new(Limits::from_config(&config));
        Ok(Self {
            limiter: Mutex::new(ResourceLimiter::new(config.capacity)),
            sandbox,
            telemetry: Telemetry::new(),
            events: Mutex::new(EventLog::new()),
            config,
            store,
        })
    }
    pub fn config(&self) -> &WorkerConfig {
        &self.config
    }

    /// The admission policy this worker enforces at the sandbox layer.
    pub fn limits(&self) -> Limits {
        self.sandbox.limits()
    }

    /// The deadline/resource-bound guard this worker admits jobs through.
    pub fn sandbox(&self) -> Sandbox {
        self.sandbox
    }

    pub fn checkpoint(&self, job_id: &str) -> Result<Option<JobCheckpoint>, WorkerError> {
        checkpoint::load(&self.store, job_id)
    }

    /// Snapshots the current admission budget for the read-only status surface.
    /// Locking the limiter briefly yields a consistent capacity/reserved/available
    /// triple without affecting in-flight admission.
    pub fn admission(&self) -> AdmissionSnapshot {
        let limiter = self.limiter.lock().expect("worker resource limiter mutex poisoned");
        resources::snapshot(&limiter)
    }

    /// The readiness snapshot the HTTP `/health` surface renders.
    pub fn health(&self) -> HealthSnapshot {
        HealthSnapshot::new(self.admission(), self.config.maximum_checkpoint_bytes)
    }

    /// A copy of the worker's execution counters.
    pub fn telemetry(&self) -> TelemetrySnapshot {
        self.telemetry.snapshot()
    }

    /// The recorded lifecycle events for one job, in emission order.
    pub fn events(&self, job_id: &str) -> Vec<JobEvent> {
        self.events.lock().expect("worker event log mutex poisoned").events_for(job_id)
    }

    /// Returns the ids of every job for which a durable checkpoint exists,
    /// read from the object store rather than any in-memory index.
    pub fn known_checkpoints(&self) -> Result<Vec<String>, WorkerError> {
        checkpoint::list(&self.store)
    }

    /// Decides how an interrupted job should be recovered from its durable
    /// checkpoint alone.
    pub fn recovery_plan(&self, job_id: &str) -> Result<RecoveryPlan, WorkerError> {
        recovery::plan(&self.store, job_id)
    }

    /// Deletes a job's per-job scratch objects, returning how many were removed.
    pub fn cleanup_scratch(&self, job_id: &str) -> Result<CleanupReport, WorkerError> {
        cleanup::cleanup(&self.store, job_id)
    }

    /// Records a completed job's output manifest as a checksum-verified artifact.
    pub fn record_artifact(
        &self,
        job_id: &str,
        output: &JobOutput,
    ) -> Result<ArtifactReceipt, WorkerError> {
        let receipt =
            artifacts::record(&self.store, job_id, output, self.config.maximum_checkpoint_bytes)?;
        self.telemetry.record_artifact();
        Ok(receipt)
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
        if let Err(error) = self.sandbox.admit(&envelope.work, now_ms) {
            return self.reject(envelope, now_ms, error);
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
        if let Err(error) = resources::reserve(&mut limiter, envelope.work.resources) {
            return self.reject(envelope, now_ms, error);
        }
        let result = self.run_reserved(envelope, cancellation, now_ms);
        let release = resources::release(&mut limiter, envelope.work.resources);
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
        let result = execute::run(&envelope.job, cancellation);
        let (state, detail) = match &result {
            Ok(output) => (CheckpointState::Completed, execute::output_summary(output)),
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
        )?;
        // The durable write is authoritative; in-memory telemetry and the event
        // log are updated only after it succeeds, so they never lead the store.
        self.telemetry.record_state(state);
        self.events.lock().expect("worker event log mutex poisoned").emit(
            &envelope.work.id,
            state,
            detail,
            now_ms,
        );
        Ok(())
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
