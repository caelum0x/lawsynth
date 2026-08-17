//! Append-only job lifecycle events with a monotonic sequence.
//!
//! Each durable checkpoint transition also emits an in-memory [`JobEvent`]
//! carrying a strictly increasing sequence number. This mirrors the append-only
//! run-event stream from the production architecture (sections 10 and 23): event
//! ordering is monotonic, so a consumer can replay from the last sequence it saw
//! without gaps or reordering. The log is deterministic -- sequence and recorded
//! time are supplied by the caller, never read from a hidden clock.

use crate::CheckpointState;

/// One observed lifecycle transition for a job.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobEvent {
    /// Strictly increasing across the whole log; the replay cursor.
    pub sequence: u64,
    pub job_id: String,
    pub recorded_at_ms: u64,
    pub state: CheckpointState,
    pub detail: String,
}

/// An append-only, monotonic event log.
#[derive(Clone, Debug, Default)]
pub struct EventLog {
    next_sequence: u64,
    events: Vec<JobEvent>,
}

impl EventLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an event and returns it with its assigned sequence. Sequences
    /// start at one and never repeat, so `sequence` is a total order over the log.
    pub(crate) fn emit(
        &mut self,
        job_id: &str,
        state: CheckpointState,
        detail: &str,
        recorded_at_ms: u64,
    ) -> JobEvent {
        self.next_sequence += 1;
        let event = JobEvent {
            sequence: self.next_sequence,
            job_id: job_id.to_owned(),
            recorded_at_ms,
            state,
            detail: detail.to_owned(),
        };
        self.events.push(event.clone());
        event
    }

    /// Every event for one job, in emission order.
    pub fn events_for(&self, job_id: &str) -> Vec<JobEvent> {
        self.events.iter().filter(|event| event.job_id == job_id).cloned().collect()
    }

    /// Every event whose sequence is strictly greater than `cursor`, for replay.
    pub fn since(&self, cursor: u64) -> Vec<JobEvent> {
        self.events.iter().filter(|event| event.sequence > cursor).cloned().collect()
    }

    /// The highest sequence emitted so far, or zero if the log is empty.
    pub fn latest_sequence(&self) -> u64 {
        self.next_sequence
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assigns_strictly_increasing_sequences_and_filters_by_job() {
        let mut log = EventLog::new();
        assert!(log.is_empty());
        let first = log.emit("job-a", CheckpointState::Running, "admitted", 10);
        let second = log.emit("job-b", CheckpointState::Running, "admitted", 11);
        let third = log.emit("job-a", CheckpointState::Completed, "done", 12);
        assert_eq!((first.sequence, second.sequence, third.sequence), (1, 2, 3));
        assert_eq!(log.latest_sequence(), 3);

        let job_a = log.events_for("job-a");
        assert_eq!(job_a.len(), 2);
        assert_eq!(job_a[0].state, CheckpointState::Running);
        assert_eq!(job_a[1].state, CheckpointState::Completed);
    }

    #[test]
    fn since_returns_only_events_after_the_cursor() {
        let mut log = EventLog::new();
        log.emit("job-a", CheckpointState::Running, "admitted", 10);
        log.emit("job-a", CheckpointState::Completed, "done", 12);
        let replay = log.since(1);
        assert_eq!(replay.len(), 1);
        assert_eq!(replay[0].sequence, 2);
        assert!(log.since(2).is_empty());
    }
}
