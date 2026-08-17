//! The scheduler core: queue state, placement, lease fencing, expiry recovery,
//! cancellation, and dead-letter transitions.
//!
//! This module orchestrates the lifecycle but delegates each self-contained
//! decision to a focused module: [`crate::queue`] and [`crate::priority`] pick
//! the next job, [`crate::placement`] gates admission, [`crate::pool`] and
//! [`crate::quota`] account for resources, [`crate::policy`] and
//! [`crate::recovery`] decide retry vs. dead-letter, and [`crate::database`]
//! persists checkpoints. Every real transition also updates the in-process
//! [`crate::metrics`] counters and appends to the [`crate::events`] log, giving
//! operators observable lifecycle signals without changing the durable format.

use std::collections::BTreeMap;

use lawsynth_store::ObjectStore;
use lawsynth_worker::JobEnvelope;

use crate::database::CheckpointStore;
use crate::events::{EventLog, JobEvent};
use crate::metrics::{MetricsSnapshot, SchedulerMetrics};
use crate::placement;
use crate::policy::{FailureAction, RetryPolicy};
use crate::pool::PoolRegistry;
use crate::priority::Candidate;
use crate::queue;
use crate::recovery::{self, RecoveryOutcome};
use crate::{HealthSnapshot, Lease, LeaseToken, SchedulerConfig, SchedulerError, WorkerPool};

/// The authoritative lifecycle state of a submitted job.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JobState {
    Queued,
    Leased { worker_id: String, generation: u64, expires_at_ms: u64 },
    Completed,
    Cancelled { reason: String },
    DeadLetter { reason: String },
}

impl JobState {
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled { .. } | Self::DeadLetter { .. })
    }
    fn name(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Leased { .. } => "leased",
            Self::Completed => "completed",
            Self::Cancelled { .. } => "cancelled",
            Self::DeadLetter { .. } => "dead_letter",
        }
    }
}

/// Portable record of scheduler-owned lifecycle state. The executable typed
/// payload is intentionally not serialized: no lossy job codec is available.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistedCheckpoint {
    pub job_id: String,
    pub attempt: u32,
    pub sequence: u64,
    pub updated_at_ms: u64,
    pub state: JobState,
}

#[derive(Clone, Debug)]
struct JobRecord {
    envelope: JobEnvelope,
    state: JobState,
    sequence: u64,
    generation: u64,
}

/// A synchronous scheduler with durable lifecycle checkpoints. It is safe to
/// recreate operational state from a submission source, while the object store
/// preserves audit/recovery evidence. It does not claim broker-backed recovery
/// because `JobEnvelope` deliberately has no wire codec.
pub struct Scheduler<S> {
    config: SchedulerConfig,
    checkpoints: CheckpointStore<S>,
    jobs: BTreeMap<String, JobRecord>,
    pools: PoolRegistry,
    policy: RetryPolicy,
    metrics: SchedulerMetrics,
    events: EventLog,
}

impl<S: ObjectStore> Scheduler<S> {
    pub fn new(config: SchedulerConfig, store: S) -> Result<Self, SchedulerError> {
        let policy = RetryPolicy::new(config.maximum_attempts);
        let checkpoints = CheckpointStore::new(store, config.maximum_checkpoint_bytes);
        Ok(Self {
            config,
            checkpoints,
            jobs: BTreeMap::new(),
            pools: PoolRegistry::new(),
            policy,
            metrics: SchedulerMetrics::new(),
            events: EventLog::default(),
        })
    }

    pub fn config(&self) -> &SchedulerConfig {
        &self.config
    }

    /// An immutable snapshot of the lifecycle counters.
    pub fn metrics(&self) -> MetricsSnapshot {
        self.metrics.snapshot()
    }

    /// The append-only lifecycle event log.
    pub fn events(&self) -> &EventLog {
        &self.events
    }

    /// A readiness and health view for the control-plane `/health` route.
    pub fn health_snapshot(&self) -> HealthSnapshot {
        HealthSnapshot::new(
            self.queued_count(),
            self.config.maximum_queued_jobs,
            self.config.maximum_attempts,
            duration_ms(self.config.lease_duration),
            self.config.maximum_checkpoint_bytes,
            self.metrics.snapshot(),
        )
    }

    pub fn register_pool(&mut self, pool: WorkerPool) -> Result<(), SchedulerError> {
        self.pools.register(pool)
    }

    pub fn submit(&mut self, envelope: JobEnvelope, now_ms: u64) -> Result<(), SchedulerError> {
        let id = envelope.work.id.clone();
        if self.jobs.contains_key(&id) {
            return Err(SchedulerError::DuplicateJob(id));
        }
        if self.queued_count() >= self.config.maximum_queued_jobs {
            return Err(SchedulerError::QueueFull { limit: self.config.maximum_queued_jobs });
        }
        let record = JobRecord { envelope, state: JobState::Queued, sequence: 1, generation: 0 };
        self.write_checkpoint(&id, &record, now_ms)?;
        self.jobs.insert(id.clone(), record);
        self.metrics.record_queued();
        self.events.emit(now_ms, id, JobEvent::Queued);
        Ok(())
    }

    /// Recovers abandoned leases, then selects the oldest-deadline compatible
    /// queued job for a worker pool. Expired jobs are never dispatched.
    pub fn lease_next(
        &mut self,
        worker_id: &str,
        now_ms: u64,
    ) -> Result<Option<Lease>, SchedulerError> {
        self.recover_expired(now_ms)?;
        let available = self.pools.available(worker_id)?;
        let candidates = self.jobs.iter().filter_map(|(id, record)| {
            let placeable = matches!(record.state, JobState::Queued)
                && placement::is_placeable(
                    record.envelope.work.resources,
                    available,
                    record.envelope.work.is_expired(now_ms),
                );
            placeable.then(|| {
                Candidate::new(
                    record.envelope.work.deadline_at_ms,
                    record.envelope.work.submitted_at_ms,
                    id.clone(),
                )
            })
        });
        let Some(candidate) = queue::select(candidates) else {
            self.expire_queued_jobs(now_ms)?;
            return Ok(None);
        };
        let id = candidate.id;

        let (resource, lease, generation) = {
            let record = self.jobs.get_mut(&id).expect("selected job exists");
            record.generation = record.generation.checked_add(1).ok_or_else(|| {
                SchedulerError::CorruptCheckpoint("lease generation overflow".into())
            })?;
            let expires_at_ms = now_ms
                .saturating_add(duration_ms(self.config.lease_duration))
                .min(record.envelope.work.deadline_at_ms);
            if expires_at_ms <= now_ms {
                return Ok(None);
            }
            record.state = JobState::Leased {
                worker_id: worker_id.to_owned(),
                generation: record.generation,
                expires_at_ms,
            };
            record.sequence = next_sequence(record.sequence)?;
            let token = LeaseToken {
                job_id: id.clone(),
                worker_id: worker_id.to_owned(),
                generation: record.generation,
            };
            (
                record.envelope.work.resources,
                Lease {
                    token,
                    issued_at_ms: now_ms,
                    expires_at_ms,
                    envelope: record.envelope.clone(),
                },
                record.generation,
            )
        };
        let checkpoint = self.jobs.get(&id).expect("leased job exists").clone();
        self.write_checkpoint(&id, &checkpoint, now_ms)?;
        self.pools.reserve(worker_id, resource)?;
        self.metrics.record_leased();
        self.events.emit(
            now_ms,
            id,
            JobEvent::Leased { worker_id: worker_id.to_owned(), generation },
        );
        Ok(Some(lease))
    }

    /// Extends a lease after validating its fencing token. The hard job deadline
    /// remains authoritative even when workers heartbeat successfully.
    pub fn heartbeat(&mut self, token: &LeaseToken, now_ms: u64) -> Result<Lease, SchedulerError> {
        let lease_duration_ms = duration_ms(self.config.lease_duration);
        let (expires_at_ms, envelope) = {
            let record = self.record_for_token_mut(token, now_ms)?;
            let expires_at_ms =
                now_ms.saturating_add(lease_duration_ms).min(record.envelope.work.deadline_at_ms);
            if expires_at_ms <= now_ms {
                return Err(SchedulerError::LeaseExpired { job_id: token.job_id.clone() });
            }
            record.state = JobState::Leased {
                worker_id: token.worker_id.clone(),
                generation: token.generation,
                expires_at_ms,
            };
            record.sequence = next_sequence(record.sequence)?;
            (expires_at_ms, record.envelope.clone())
        };
        let checkpoint = self.jobs.get(&token.job_id).expect("validated job exists").clone();
        self.write_checkpoint(&token.job_id, &checkpoint, now_ms)?;
        Ok(Lease { token: token.clone(), issued_at_ms: now_ms, expires_at_ms, envelope })
    }

    pub fn complete(&mut self, token: &LeaseToken, now_ms: u64) -> Result<(), SchedulerError> {
        let resource = {
            let record = self.record_for_token_mut(token, now_ms)?;
            record.state = JobState::Completed;
            record.sequence = next_sequence(record.sequence)?;
            record.envelope.work.resources
        };
        let checkpoint = self.jobs.get(&token.job_id).expect("validated job exists").clone();
        self.write_checkpoint(&token.job_id, &checkpoint, now_ms)?;
        self.pools.release(&token.worker_id, resource)?;
        self.metrics.record_completed();
        self.events.emit(now_ms, token.job_id.clone(), JobEvent::Completed);
        Ok(())
    }

    /// Fails an in-flight job. Retryable failures requeue with a new worker
    /// attempt; permanent failures and exhausted attempts reach dead letter.
    pub fn fail(
        &mut self,
        token: &LeaseToken,
        retryable: bool,
        reason: impl Into<String>,
        now_ms: u64,
    ) -> Result<JobState, SchedulerError> {
        let reason = bounded_reason(reason.into())?;
        let policy = self.policy;
        let (resource, state) = {
            let record = self.record_for_token_mut(token, now_ms)?;
            let attempt = record.envelope.work.attempt;
            record.state = match policy.on_failure(retryable, attempt) {
                FailureAction::Requeue => {
                    let attempt = record.envelope.work.attempt;
                    record.envelope.work.attempt = attempt.checked_add(1).ok_or_else(|| {
                        SchedulerError::CorruptCheckpoint("attempt overflow".into())
                    })?;
                    JobState::Queued
                }
                FailureAction::DeadLetter => JobState::DeadLetter { reason },
            };
            record.sequence = next_sequence(record.sequence)?;
            (record.envelope.work.resources, record.state.clone())
        };
        let checkpoint = self.jobs.get(&token.job_id).expect("validated job exists").clone();
        self.write_checkpoint(&token.job_id, &checkpoint, now_ms)?;
        self.pools.release(&token.worker_id, resource)?;
        self.metrics.record_failed();
        self.emit_state_event(&token.job_id, &state, now_ms);
        Ok(state)
    }

    /// Cancels queued or leased work. A stale worker completion is fenced by the
    /// removed lease state; actual process interruption remains the worker's
    /// cooperative cancellation responsibility.
    pub fn cancel(
        &mut self,
        job_id: &str,
        reason: impl Into<String>,
        now_ms: u64,
    ) -> Result<(), SchedulerError> {
        let reason = bounded_reason(reason.into())?;
        let (leased_worker, resource) = {
            let record = self
                .jobs
                .get_mut(job_id)
                .ok_or_else(|| SchedulerError::UnknownJob(job_id.to_owned()))?;
            if record.state.is_terminal() {
                return Err(SchedulerError::InvalidTransition {
                    job_id: job_id.to_owned(),
                    state: record.state.name(),
                });
            }
            let leased_worker = match &record.state {
                JobState::Leased { worker_id, .. } => Some(worker_id.clone()),
                JobState::Queued => None,
                _ => unreachable!("nonterminal state is queued or leased"),
            };
            let resource = record.envelope.work.resources;
            record.state = JobState::Cancelled { reason: reason.clone() };
            record.sequence = next_sequence(record.sequence)?;
            (leased_worker, resource)
        };
        let checkpoint = self.jobs.get(job_id).expect("cancelled job exists").clone();
        self.write_checkpoint(job_id, &checkpoint, now_ms)?;
        if let Some(worker_id) = leased_worker {
            self.pools.release(&worker_id, resource)?;
        }
        self.metrics.record_cancelled();
        self.events.emit(now_ms, job_id.to_owned(), JobEvent::Cancelled { reason });
        Ok(())
    }

    /// Moves expired leases back to the queue (or dead letter after their last
    /// attempt) and expires queued jobs whose deadline has already elapsed.
    pub fn recover_expired(&mut self, now_ms: u64) -> Result<usize, SchedulerError> {
        let expired = self
            .jobs
            .iter()
            .filter_map(|(id, record)| match &record.state {
                JobState::Leased { expires_at_ms, .. }
                    if *expires_at_ms <= now_ms || record.envelope.work.is_expired(now_ms) =>
                {
                    Some(id.clone())
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut recovered = 0;
        for id in expired {
            let (worker_id, resource, state) = {
                let maximum_attempts = self.config.maximum_attempts;
                let record = self.jobs.get_mut(&id).expect("expired job exists");
                let JobState::Leased { worker_id, .. } = &record.state else {
                    continue;
                };
                let worker_id = worker_id.clone();
                let resource = record.envelope.work.resources;
                let deadline_elapsed = record.envelope.work.is_expired(now_ms);
                match recovery::on_lease_expiry(
                    record.envelope.work.attempt,
                    maximum_attempts,
                    deadline_elapsed,
                ) {
                    RecoveryOutcome::Requeue => {
                        record.envelope.work.attempt += 1;
                        record.state = JobState::Queued;
                    }
                    RecoveryOutcome::DeadLetter { reason } => {
                        record.state = JobState::DeadLetter { reason };
                    }
                }
                record.sequence = next_sequence(record.sequence)?;
                (worker_id, resource, record.state.clone())
            };
            let checkpoint = self.jobs.get(&id).expect("recovered job exists").clone();
            self.write_checkpoint(&id, &checkpoint, now_ms)?;
            self.pools.release(&worker_id, resource)?;
            self.emit_state_event(&id, &state, now_ms);
            recovered += 1;
        }
        self.expire_queued_jobs(now_ms)?;
        Ok(recovered)
    }

    pub fn state(&self, job_id: &str) -> Result<&JobState, SchedulerError> {
        self.jobs
            .get(job_id)
            .map(|record| &record.state)
            .ok_or_else(|| SchedulerError::UnknownJob(job_id.to_owned()))
    }

    pub fn checkpoint(&self, job_id: &str) -> Result<Option<PersistedCheckpoint>, SchedulerError> {
        self.checkpoints.read(job_id)
    }

    pub fn queued_count(&self) -> usize {
        self.jobs.values().filter(|record| matches!(record.state, JobState::Queued)).count()
    }

    fn record_for_token_mut(
        &mut self,
        token: &LeaseToken,
        now_ms: u64,
    ) -> Result<&mut JobRecord, SchedulerError> {
        let record = self
            .jobs
            .get_mut(&token.job_id)
            .ok_or_else(|| SchedulerError::UnknownJob(token.job_id.clone()))?;
        let JobState::Leased { worker_id, generation, expires_at_ms } = &record.state else {
            return Err(SchedulerError::StaleLease { job_id: token.job_id.clone() });
        };
        if worker_id != &token.worker_id || generation != &token.generation {
            return Err(SchedulerError::StaleLease { job_id: token.job_id.clone() });
        }
        if *expires_at_ms <= now_ms || record.envelope.work.is_expired(now_ms) {
            return Err(SchedulerError::LeaseExpired { job_id: token.job_id.clone() });
        }
        Ok(record)
    }

    fn expire_queued_jobs(&mut self, now_ms: u64) -> Result<(), SchedulerError> {
        let expired = self
            .jobs
            .iter()
            .filter_map(|(id, record)| {
                (matches!(record.state, JobState::Queued)
                    && record.envelope.work.is_expired(now_ms))
                .then_some(id.clone())
            })
            .collect::<Vec<_>>();
        for id in expired {
            let reason = recovery::QUEUED_EXPIRY_REASON.to_owned();
            let record = self.jobs.get_mut(&id).expect("expired queue job exists");
            record.state = JobState::DeadLetter { reason: reason.clone() };
            record.sequence = next_sequence(record.sequence)?;
            let snapshot = record.clone();
            self.write_checkpoint(&id, &snapshot, now_ms)?;
            self.metrics.record_dead_letter();
            self.events.emit(now_ms, id, JobEvent::DeadLetter { reason });
        }
        Ok(())
    }

    /// Emits the metrics counter and event matching a post-transition state.
    fn emit_state_event(&mut self, job_id: &str, state: &JobState, now_ms: u64) {
        match state {
            JobState::Queued => {
                self.metrics.record_queued();
                self.events.emit(now_ms, job_id.to_owned(), JobEvent::Queued);
            }
            JobState::DeadLetter { reason } => {
                self.metrics.record_dead_letter();
                self.events.emit(
                    now_ms,
                    job_id.to_owned(),
                    JobEvent::DeadLetter { reason: reason.clone() },
                );
            }
            _ => {}
        }
    }

    fn write_checkpoint(
        &self,
        job_id: &str,
        record: &JobRecord,
        now_ms: u64,
    ) -> Result<(), SchedulerError> {
        let checkpoint = PersistedCheckpoint {
            job_id: job_id.to_owned(),
            attempt: record.envelope.work.attempt,
            sequence: record.sequence,
            updated_at_ms: now_ms,
            state: record.state.clone(),
        };
        self.checkpoints.write(&checkpoint)
    }
}

fn duration_ms(duration: std::time::Duration) -> u64 {
    duration.as_millis().try_into().unwrap_or(u64::MAX)
}

fn next_sequence(sequence: u64) -> Result<u64, SchedulerError> {
    sequence
        .checked_add(1)
        .ok_or_else(|| SchedulerError::CorruptCheckpoint("checkpoint sequence overflow".into()))
}

fn bounded_reason(reason: String) -> Result<String, SchedulerError> {
    if reason.is_empty() || reason.len() > 2_048 || reason.contains('\n') || reason.contains('\r') {
        return Err(SchedulerError::InvalidConfig(
            "lifecycle reason must be 1..=2048 bytes and single-line".into(),
        ));
    }
    Ok(reason)
}
