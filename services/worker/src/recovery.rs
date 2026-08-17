//! Checkpoint-based recovery of an interrupted job.
//!
//! A worker may be lost mid-job. Its only durable trace is the lifecycle
//! checkpoint chain in the object store, so recovery is decided from that record
//! alone. The production architecture is explicit that "checkpoint-compatible
//! jobs resume, others restart explicitly" (section 23). This worker's executable
//! payloads are typed and in-memory -- they are deliberately never serialized --
//! so a genuine payload *resume* is not possible; instead recovery classifies
//! what the durable record proves and hands the caller an explicit decision.

use lawsynth_store::ObjectStore;

use crate::{CheckpointState, WorkerError, checkpoint};

/// The recovery decision for a job, derived from its durable checkpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryAction {
    /// No durable record exists; run the job normally.
    Fresh,
    /// A non-terminal record exists: a previous attempt was admitted but never
    /// reached a terminal state, so it was interrupted. Because payloads are not
    /// serialized, the job must be re-executed explicitly rather than resumed.
    Reexecute,
    /// A terminal record exists; the job is finished and must not run again.
    AlreadyFinished,
}

/// A recovery plan: the decision plus the durable facts it was derived from.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryPlan {
    pub job_id: String,
    pub action: RecoveryAction,
    /// The last durable state, if any record exists.
    pub last_state: Option<CheckpointState>,
    /// The sequence of the last durable checkpoint, or zero if none exists.
    pub last_sequence: u64,
}

impl RecoveryPlan {
    /// Whether the job may be (re)dispatched under this plan.
    pub fn is_runnable(&self) -> bool {
        matches!(self.action, RecoveryAction::Fresh | RecoveryAction::Reexecute)
    }
}

/// Builds a recovery plan for `job_id` by reading its durable checkpoint. A
/// corrupt checkpoint is surfaced as an error rather than silently treated as a
/// fresh job, so integrity faults never cause duplicate execution.
pub(crate) fn plan<S: ObjectStore>(store: &S, job_id: &str) -> Result<RecoveryPlan, WorkerError> {
    match checkpoint::load(store, job_id)? {
        None => Ok(RecoveryPlan {
            job_id: job_id.to_owned(),
            action: RecoveryAction::Fresh,
            last_state: None,
            last_sequence: 0,
        }),
        Some(record) => {
            let action = if record.state.is_terminal() {
                RecoveryAction::AlreadyFinished
            } else {
                RecoveryAction::Reexecute
            };
            Ok(RecoveryPlan {
                job_id: job_id.to_owned(),
                action,
                last_state: Some(record.state),
                last_sequence: record.sequence,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::JobCheckpoint;
    use lawsynth_store::MemoryStore;

    fn save(store: &MemoryStore, job_id: &str, state: CheckpointState, sequence: u64) {
        checkpoint::save(
            store,
            &JobCheckpoint {
                job_id: job_id.to_owned(),
                sequence,
                recorded_at_ms: 10,
                state,
                detail: "detail".into(),
            },
            1024,
        )
        .unwrap();
    }

    #[test]
    fn absent_record_is_a_fresh_run() {
        let store = MemoryStore::default();
        let plan = plan(&store, "job-a").unwrap();
        assert_eq!(plan.action, RecoveryAction::Fresh);
        assert_eq!(plan.last_sequence, 0);
        assert!(plan.is_runnable());
    }

    #[test]
    fn interrupted_running_record_is_reexecuted() {
        let store = MemoryStore::default();
        save(&store, "job-a", CheckpointState::Running, 1);
        let plan = plan(&store, "job-a").unwrap();
        assert_eq!(plan.action, RecoveryAction::Reexecute);
        assert_eq!(plan.last_state, Some(CheckpointState::Running));
        assert_eq!(plan.last_sequence, 1);
        assert!(plan.is_runnable());
    }

    #[test]
    fn terminal_record_is_already_finished() {
        let store = MemoryStore::default();
        save(&store, "job-a", CheckpointState::Completed, 2);
        let plan = plan(&store, "job-a").unwrap();
        assert_eq!(plan.action, RecoveryAction::AlreadyFinished);
        assert!(!plan.is_runnable());
    }
}
