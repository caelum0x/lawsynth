//! In-process lifecycle event log with a monotonic sequence.
//!
//! Every scheduler state transition emits a [`JobEvent`] into an append-only
//! [`EventLog`]. Each record carries a strictly increasing `sequence`, satisfying
//! the architecture's "append-only run events" and "monotonic per run attempt"
//! ordering guarantees, so a consumer (Studio, a broker seam) could replay from
//! its last seen sequence. The log is a bounded ring buffer: when it reaches
//! capacity the oldest record is dropped, but the sequence counter never resets,
//! so replay position is always well defined.

use std::collections::VecDeque;

/// The lifecycle transition an event records.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JobEvent {
    /// The job entered the queue (submission or retry requeue).
    Queued,
    /// The job was leased to a worker.
    Leased { worker_id: String, generation: u64 },
    /// The job completed successfully.
    Completed,
    /// The job was cancelled by the control plane.
    Cancelled { reason: String },
    /// The job reached the dead-letter terminal state.
    DeadLetter { reason: String },
}

impl JobEvent {
    /// A stable, lower-case name for rendering and filtering.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Leased { .. } => "leased",
            Self::Completed => "completed",
            Self::Cancelled { .. } => "cancelled",
            Self::DeadLetter { .. } => "dead_letter",
        }
    }
}

/// One recorded lifecycle event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventRecord {
    pub sequence: u64,
    pub at_ms: u64,
    pub job_id: String,
    pub event: JobEvent,
}

/// A bounded, append-only log of lifecycle events with monotonic sequencing.
#[derive(Clone, Debug)]
pub struct EventLog {
    records: VecDeque<EventRecord>,
    capacity: usize,
    next_sequence: u64,
}

impl EventLog {
    /// Builds a log retaining at most `capacity` recent records (minimum one).
    pub fn new(capacity: usize) -> Self {
        Self { records: VecDeque::new(), capacity: capacity.max(1), next_sequence: 1 }
    }

    /// Appends an event, assigning and returning its sequence number.
    pub fn emit(&mut self, at_ms: u64, job_id: impl Into<String>, event: JobEvent) -> u64 {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        if self.records.len() == self.capacity {
            self.records.pop_front();
        }
        self.records.push_back(EventRecord { sequence, at_ms, job_id: job_id.into(), event });
        sequence
    }

    /// The retained records, oldest first.
    pub fn records(&self) -> impl Iterator<Item = &EventRecord> {
        self.records.iter()
    }

    /// Records strictly newer than `sequence`, for replay from a known position.
    pub fn since(&self, sequence: u64) -> impl Iterator<Item = &EventRecord> {
        self.records.iter().filter(move |record| record.sequence > sequence)
    }

    /// The sequence that will be assigned to the next emitted event.
    pub fn next_sequence(&self) -> u64 {
        self.next_sequence
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl Default for EventLog {
    fn default() -> Self {
        Self::new(1_024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assigns_monotonic_sequences() {
        let mut log = EventLog::new(8);
        assert_eq!(log.emit(1, "job-1", JobEvent::Queued), 1);
        assert_eq!(
            log.emit(2, "job-1", JobEvent::Leased { worker_id: "w".into(), generation: 1 }),
            2
        );
        assert_eq!(log.emit(3, "job-1", JobEvent::Completed), 3);
        assert_eq!(log.len(), 3);
    }

    #[test]
    fn since_replays_from_a_known_position() {
        let mut log = EventLog::new(8);
        log.emit(1, "a", JobEvent::Queued);
        log.emit(2, "b", JobEvent::Queued);
        log.emit(3, "c", JobEvent::Queued);
        let tail: Vec<&str> = log.since(1).map(|record| record.job_id.as_str()).collect();
        assert_eq!(tail, vec!["b", "c"]);
    }

    #[test]
    fn drops_oldest_but_keeps_sequence_monotonic() {
        let mut log = EventLog::new(2);
        log.emit(1, "a", JobEvent::Queued);
        log.emit(2, "b", JobEvent::Queued);
        let third = log.emit(3, "c", JobEvent::Queued);
        assert_eq!(third, 3);
        assert_eq!(log.len(), 2);
        let ids: Vec<&str> = log.records().map(|record| record.job_id.as_str()).collect();
        assert_eq!(ids, vec!["b", "c"]);
    }
}
