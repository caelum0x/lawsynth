use std::collections::BTreeMap;

use lawsynth_runner::ResourceRequest;
use lawsynth_store::{ObjectKey, ObjectStore, StoreError};
use lawsynth_worker::JobEnvelope;

use crate::{Lease, LeaseToken, SchedulerConfig, SchedulerError, WorkerPool};

const CHECKPOINT_PREFIX: &str = "scheduler/checkpoints/";

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

#[derive(Clone, Debug)]
struct PoolState {
    pool: WorkerPool,
    reserved: ResourceRequest,
}

/// A synchronous scheduler with durable lifecycle checkpoints. It is safe to
/// recreate operational state from a submission source, while the object store
/// preserves audit/recovery evidence. It does not claim broker-backed recovery
/// because `JobEnvelope` deliberately has no wire codec.
pub struct Scheduler<S> {
    config: SchedulerConfig,
    store: S,
    jobs: BTreeMap<String, JobRecord>,
    pools: BTreeMap<String, PoolState>,
}

impl<S: ObjectStore> Scheduler<S> {
    pub fn new(config: SchedulerConfig, store: S) -> Result<Self, SchedulerError> {
        Ok(Self { config, store, jobs: BTreeMap::new(), pools: BTreeMap::new() })
    }

    pub fn config(&self) -> &SchedulerConfig {
        &self.config
    }

    pub fn register_pool(&mut self, pool: WorkerPool) -> Result<(), SchedulerError> {
        if self.pools.contains_key(&pool.id) {
            return Err(SchedulerError::InvalidWorker(format!(
                "pool '{}' is already registered",
                pool.id
            )));
        }
        self.pools.insert(pool.id.clone(), PoolState { pool, reserved: zero_resources() });
        Ok(())
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
        self.jobs.insert(id, record);
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
        let available = self.available_resources(worker_id)?;
        let selected = self
            .jobs
            .iter()
            .filter_map(|(id, record)| {
                (matches!(record.state, JobState::Queued)
                    && !record.envelope.work.is_expired(now_ms)
                    && record.envelope.work.resources.fits_within(available))
                .then_some((
                    record.envelope.work.deadline_at_ms,
                    record.envelope.work.submitted_at_ms,
                    id.clone(),
                ))
            })
            .min();
        let Some((_, _, id)) = selected else {
            self.expire_queued_jobs(now_ms)?;
            return Ok(None);
        };

        let (resource, lease) = {
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
            )
        };
        let checkpoint = self.jobs.get(&id).expect("leased job exists").clone();
        self.write_checkpoint(&id, &checkpoint, now_ms)?;
        self.reserve(worker_id, resource)?;
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
        self.release(&token.worker_id, resource)
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
        let (resource, state) = {
            let maximum_attempts = self.config.maximum_attempts;
            let record = self.record_for_token_mut(token, now_ms)?;
            let attempt = record.envelope.work.attempt;
            record.state = if retryable && attempt < maximum_attempts {
                record.envelope.work.attempt = attempt
                    .checked_add(1)
                    .ok_or_else(|| SchedulerError::CorruptCheckpoint("attempt overflow".into()))?;
                JobState::Queued
            } else {
                JobState::DeadLetter { reason }
            };
            record.sequence = next_sequence(record.sequence)?;
            (record.envelope.work.resources, record.state.clone())
        };
        let checkpoint = self.jobs.get(&token.job_id).expect("validated job exists").clone();
        self.write_checkpoint(&token.job_id, &checkpoint, now_ms)?;
        self.release(&token.worker_id, resource)?;
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
            record.state = JobState::Cancelled { reason };
            record.sequence = next_sequence(record.sequence)?;
            (leased_worker, resource)
        };
        let checkpoint = self.jobs.get(job_id).expect("cancelled job exists").clone();
        self.write_checkpoint(job_id, &checkpoint, now_ms)?;
        if let Some(worker_id) = leased_worker {
            self.release(&worker_id, resource)?;
        }
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
            let (worker_id, resource) = {
                let record = self.jobs.get_mut(&id).expect("expired job exists");
                let JobState::Leased { worker_id, .. } = &record.state else {
                    continue;
                };
                let worker_id = worker_id.clone();
                let resource = record.envelope.work.resources;
                let reason = if record.envelope.work.is_expired(now_ms) {
                    "job deadline elapsed while leased".to_owned()
                } else {
                    "worker lease expired without a heartbeat".to_owned()
                };
                if record.envelope.work.attempt < self.config.maximum_attempts
                    && !record.envelope.work.is_expired(now_ms)
                {
                    record.envelope.work.attempt += 1;
                    record.state = JobState::Queued;
                } else {
                    record.state = JobState::DeadLetter { reason };
                }
                record.sequence = next_sequence(record.sequence)?;
                (worker_id, resource)
            };
            let checkpoint = self.jobs.get(&id).expect("recovered job exists").clone();
            self.write_checkpoint(&id, &checkpoint, now_ms)?;
            self.release(&worker_id, resource)?;
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
        let key = checkpoint_key(job_id)?;
        match self.store.get(&key) {
            Ok(object) => decode_checkpoint(&object.bytes).map(Some),
            Err(StoreError::NotFound(_)) => Ok(None),
            Err(error) => Err(error.into()),
        }
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

    fn available_resources(&self, worker_id: &str) -> Result<ResourceRequest, SchedulerError> {
        let pool = self
            .pools
            .get(worker_id)
            .ok_or_else(|| SchedulerError::UnknownWorker(worker_id.to_owned()))?;
        pool.pool.capacity.checked_sub(pool.reserved).ok_or_else(|| {
            SchedulerError::CorruptCheckpoint(format!(
                "pool '{worker_id}' reserved beyond capacity"
            ))
        })
    }

    fn reserve(
        &mut self,
        worker_id: &str,
        resource: ResourceRequest,
    ) -> Result<(), SchedulerError> {
        let pool = self
            .pools
            .get_mut(worker_id)
            .ok_or_else(|| SchedulerError::UnknownWorker(worker_id.to_owned()))?;
        let reserved = pool.reserved.checked_add(resource).ok_or_else(|| {
            SchedulerError::CorruptCheckpoint("resource reservation overflow".into())
        })?;
        if !reserved.fits_within(pool.pool.capacity) {
            return Err(SchedulerError::InvalidWorker(format!(
                "pool '{worker_id}' cannot reserve assigned job"
            )));
        }
        pool.reserved = reserved;
        Ok(())
    }

    fn release(
        &mut self,
        worker_id: &str,
        resource: ResourceRequest,
    ) -> Result<(), SchedulerError> {
        let pool = self
            .pools
            .get_mut(worker_id)
            .ok_or_else(|| SchedulerError::UnknownWorker(worker_id.to_owned()))?;
        pool.reserved = pool.reserved.checked_sub(resource).ok_or_else(|| {
            SchedulerError::CorruptCheckpoint(format!(
                "pool '{worker_id}' released unreserved resources"
            ))
        })?;
        Ok(())
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
            let record = self.jobs.get_mut(&id).expect("expired queue job exists");
            record.state =
                JobState::DeadLetter { reason: "job deadline elapsed before assignment".into() };
            record.sequence = next_sequence(record.sequence)?;
            let snapshot = record.clone();
            self.write_checkpoint(&id, &snapshot, now_ms)?;
        }
        Ok(())
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
        let bytes = encode_checkpoint(&checkpoint);
        if bytes.len() > self.config.maximum_checkpoint_bytes {
            return Err(SchedulerError::CheckpointTooLarge {
                actual: bytes.len(),
                limit: self.config.maximum_checkpoint_bytes,
            });
        }
        self.store.put(checkpoint_key(job_id)?, bytes)?;
        Ok(())
    }
}

fn zero_resources() -> ResourceRequest {
    ResourceRequest { cpu_millis: 0, memory_bytes: 0, disk_bytes: 0 }
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

fn checkpoint_key(job_id: &str) -> Result<ObjectKey, SchedulerError> {
    Ok(ObjectKey::new(format!("{CHECKPOINT_PREFIX}{job_id}.state"))?)
}

fn encode_checkpoint(checkpoint: &PersistedCheckpoint) -> Vec<u8> {
    let (state, worker, generation, expires_at_ms, reason) = match &checkpoint.state {
        JobState::Queued => ("queued", "", 0, 0, ""),
        JobState::Leased { worker_id, generation, expires_at_ms } => {
            ("leased", worker_id.as_str(), *generation, *expires_at_ms, "")
        }
        JobState::Completed => ("completed", "", 0, 0, ""),
        JobState::Cancelled { reason } => ("cancelled", "", 0, 0, reason.as_str()),
        JobState::DeadLetter { reason } => ("dead_letter", "", 0, 0, reason.as_str()),
    };
    format!(
        "version=1\njob_id={}\nattempt={}\nsequence={}\nupdated_at_ms={}\nstate={state}\nworker={}\ngeneration={generation}\nexpires_at_ms={expires_at_ms}\nreason={}\n",
        checkpoint.job_id,
        checkpoint.attempt,
        checkpoint.sequence,
        checkpoint.updated_at_ms,
        hex(worker),
        hex(reason),
    )
    .into_bytes()
}

fn decode_checkpoint(bytes: &[u8]) -> Result<PersistedCheckpoint, SchedulerError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| SchedulerError::CorruptCheckpoint("checkpoint is not UTF-8".into()))?;
    let mut fields = BTreeMap::new();
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            return Err(SchedulerError::CorruptCheckpoint("malformed checkpoint line".into()));
        };
        if fields.insert(key, value).is_some() {
            return Err(SchedulerError::CorruptCheckpoint("duplicate checkpoint field".into()));
        }
    }
    if fields.get("version") != Some(&"1") || fields.len() != 10 {
        return Err(SchedulerError::CorruptCheckpoint(
            "unsupported checkpoint version or fields".into(),
        ));
    }
    let job_id = required(&fields, "job_id")?.to_owned();
    let attempt = number(&fields, "attempt")?;
    if attempt == 0 {
        return Err(SchedulerError::CorruptCheckpoint("attempt is zero".into()));
    }
    let sequence = number(&fields, "sequence")?;
    let updated_at_ms = number(&fields, "updated_at_ms")?;
    let worker = unhex(required(&fields, "worker")?)?;
    let generation = number(&fields, "generation")?;
    let expires_at_ms = number(&fields, "expires_at_ms")?;
    let reason = unhex(required(&fields, "reason")?)?;
    let state = match required(&fields, "state")? {
        "queued"
            if worker.is_empty() && generation == 0 && expires_at_ms == 0 && reason.is_empty() =>
        {
            JobState::Queued
        }
        "leased"
            if !worker.is_empty() && generation > 0 && expires_at_ms > 0 && reason.is_empty() =>
        {
            JobState::Leased { worker_id: worker, generation, expires_at_ms }
        }
        "completed"
            if worker.is_empty() && generation == 0 && expires_at_ms == 0 && reason.is_empty() =>
        {
            JobState::Completed
        }
        "cancelled"
            if worker.is_empty() && generation == 0 && expires_at_ms == 0 && !reason.is_empty() =>
        {
            JobState::Cancelled { reason }
        }
        "dead_letter"
            if worker.is_empty() && generation == 0 && expires_at_ms == 0 && !reason.is_empty() =>
        {
            JobState::DeadLetter { reason }
        }
        _ => return Err(SchedulerError::CorruptCheckpoint("inconsistent state fields".into())),
    };
    Ok(PersistedCheckpoint { job_id, attempt, sequence, updated_at_ms, state })
}

fn required<'a>(fields: &'a BTreeMap<&str, &str>, key: &str) -> Result<&'a str, SchedulerError> {
    fields
        .get(key)
        .copied()
        .ok_or_else(|| SchedulerError::CorruptCheckpoint(format!("missing '{key}'")))
}

fn number<T: std::str::FromStr>(
    fields: &BTreeMap<&str, &str>,
    key: &str,
) -> Result<T, SchedulerError> {
    required(fields, key)?
        .parse()
        .map_err(|_| SchedulerError::CorruptCheckpoint(format!("invalid '{key}'")))
}

fn hex(value: &str) -> String {
    value.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unhex(value: &str) -> Result<String, SchedulerError> {
    if value.len() % 2 != 0 {
        return Err(SchedulerError::CorruptCheckpoint("odd hex field".into()));
    }
    let bytes = (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16)
                .map_err(|_| SchedulerError::CorruptCheckpoint("invalid hex field".into()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    String::from_utf8(bytes)
        .map_err(|_| SchedulerError::CorruptCheckpoint("text field is not UTF-8".into()))
}
