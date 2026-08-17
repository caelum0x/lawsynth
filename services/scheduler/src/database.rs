//! Durable checkpoint persistence over an [`ObjectStore`].
//!
//! This module formalizes the scheduler's lifecycle-checkpoint I/O behind a small
//! [`CheckpointStore`] abstraction. It owns the on-disk format — a versioned,
//! line-oriented record with hex-encoded free-text fields — plus the size ceiling
//! and the object-key layout. The executable typed `JobEnvelope` payload is
//! never serialized here; only scheduler-owned lifecycle state is, exactly as the
//! architecture requires ("scheduler is reconstructable from database state").

use std::collections::BTreeMap;

use lawsynth_store::{ObjectKey, ObjectStore, StoreError};

use crate::{JobState, PersistedCheckpoint, SchedulerError};

/// Object-key prefix under which every job's latest checkpoint is written.
const CHECKPOINT_PREFIX: &str = "scheduler/checkpoints/";

/// Durable store of the latest [`PersistedCheckpoint`] per job.
///
/// A single object key per job holds its most recent lifecycle state; writes are
/// last-writer-wins and bounded by `maximum_bytes` so a runaway record cannot be
/// persisted.
pub struct CheckpointStore<S> {
    store: S,
    maximum_bytes: usize,
}

impl<S: ObjectStore> CheckpointStore<S> {
    pub fn new(store: S, maximum_bytes: usize) -> Self {
        Self { store, maximum_bytes }
    }

    /// Borrows the underlying object store (audit/recovery tooling).
    pub fn store(&self) -> &S {
        &self.store
    }

    /// Persists a checkpoint, refusing a record larger than the configured ceiling.
    pub fn write(&self, checkpoint: &PersistedCheckpoint) -> Result<(), SchedulerError> {
        let bytes = encode(checkpoint);
        if bytes.len() > self.maximum_bytes {
            return Err(SchedulerError::CheckpointTooLarge {
                actual: bytes.len(),
                limit: self.maximum_bytes,
            });
        }
        self.store.put(key(&checkpoint.job_id)?, bytes)?;
        Ok(())
    }

    /// Reads a job's checkpoint, distinguishing "absent" from "corrupt".
    pub fn read(&self, job_id: &str) -> Result<Option<PersistedCheckpoint>, SchedulerError> {
        match self.store.get(&key(job_id)?) {
            Ok(object) => decode(&object.bytes).map(Some),
            Err(StoreError::NotFound(_)) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }
}

/// The object key holding a job's latest checkpoint.
pub fn key(job_id: &str) -> Result<ObjectKey, SchedulerError> {
    Ok(ObjectKey::new(format!("{CHECKPOINT_PREFIX}{job_id}.state"))?)
}

/// Encodes a checkpoint into the durable, line-oriented record format.
pub fn encode(checkpoint: &PersistedCheckpoint) -> Vec<u8> {
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

/// Decodes a durable checkpoint record, validating every field invariant.
pub fn decode(bytes: &[u8]) -> Result<PersistedCheckpoint, SchedulerError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use lawsynth_store::MemoryStore;

    fn checkpoint(state: JobState) -> PersistedCheckpoint {
        PersistedCheckpoint {
            job_id: "job-1".into(),
            attempt: 1,
            sequence: 4,
            updated_at_ms: 1_234,
            state,
        }
    }

    #[test]
    fn round_trips_every_state_through_encode_decode() {
        for state in [
            JobState::Queued,
            JobState::Leased { worker_id: "cpu-a".into(), generation: 2, expires_at_ms: 99 },
            JobState::Completed,
            JobState::Cancelled { reason: "stop".into() },
            JobState::DeadLetter { reason: "exhausted".into() },
        ] {
            let original = checkpoint(state);
            let decoded = decode(&encode(&original)).unwrap();
            assert_eq!(decoded, original);
        }
    }

    #[test]
    fn write_then_read_survives_the_object_store() {
        let store = CheckpointStore::new(MemoryStore::default(), 8192);
        let original = checkpoint(JobState::Cancelled { reason: "audit".into() });
        store.write(&original).unwrap();
        assert_eq!(store.read("job-1").unwrap().unwrap(), original);
        assert_eq!(store.read("absent").unwrap(), None);
    }

    #[test]
    fn rejects_a_record_over_the_ceiling() {
        let store = CheckpointStore::new(MemoryStore::default(), 16);
        let error = store.write(&checkpoint(JobState::Queued)).unwrap_err();
        assert!(matches!(error, SchedulerError::CheckpointTooLarge { .. }));
    }

    #[test]
    fn decode_rejects_corrupt_bytes() {
        assert!(matches!(decode(b"not-a-checkpoint"), Err(SchedulerError::CorruptCheckpoint(_))));
    }
}
