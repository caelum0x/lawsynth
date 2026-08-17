//! Lightweight, thread-safe execution counters.
//!
//! The worker records how many jobs reached each lifecycle state, plus how many
//! artifacts it handed off and heartbeats it emitted. Counters are plain atomics
//! so the shared, `Arc`-held worker can update them from any connection thread
//! without locking. [`Telemetry::snapshot`] takes a coherent-enough copy for the
//! status surface; it is monotonic and never blocks execution.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::CheckpointState;

/// Monotonic counters describing a worker's execution history.
#[derive(Debug, Default)]
pub struct Telemetry {
    admitted: AtomicU64,
    completed: AtomicU64,
    failed: AtomicU64,
    cancelled: AtomicU64,
    rejected: AtomicU64,
    artifacts_uploaded: AtomicU64,
    heartbeats: AtomicU64,
}

impl Telemetry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records one lifecycle transition. A job that runs normally records an
    /// `admitted` (its `Running` write) and then a terminal counter, so the
    /// counters describe transitions rather than distinct jobs.
    pub(crate) fn record_state(&self, state: CheckpointState) {
        let counter = match state {
            CheckpointState::Running => &self.admitted,
            CheckpointState::Completed => &self.completed,
            CheckpointState::Failed => &self.failed,
            CheckpointState::Cancelled => &self.cancelled,
            CheckpointState::Rejected => &self.rejected,
        };
        counter.fetch_add(1, Ordering::Relaxed);
    }

    /// Records a successful artifact handoff.
    pub(crate) fn record_artifact(&self) {
        self.artifacts_uploaded.fetch_add(1, Ordering::Relaxed);
    }

    /// Records an emitted lease heartbeat.
    pub fn record_heartbeat(&self) {
        self.heartbeats.fetch_add(1, Ordering::Relaxed);
    }

    /// Takes a copy of every counter for reporting.
    pub fn snapshot(&self) -> TelemetrySnapshot {
        TelemetrySnapshot {
            admitted: self.admitted.load(Ordering::Relaxed),
            completed: self.completed.load(Ordering::Relaxed),
            failed: self.failed.load(Ordering::Relaxed),
            cancelled: self.cancelled.load(Ordering::Relaxed),
            rejected: self.rejected.load(Ordering::Relaxed),
            artifacts_uploaded: self.artifacts_uploaded.load(Ordering::Relaxed),
            heartbeats: self.heartbeats.load(Ordering::Relaxed),
        }
    }
}

/// An immutable snapshot of the worker's counters.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TelemetrySnapshot {
    pub admitted: u64,
    pub completed: u64,
    pub failed: u64,
    pub cancelled: u64,
    pub rejected: u64,
    pub artifacts_uploaded: u64,
    pub heartbeats: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_each_lifecycle_transition_and_artifact() {
        let telemetry = Telemetry::new();
        telemetry.record_state(CheckpointState::Running);
        telemetry.record_state(CheckpointState::Running);
        telemetry.record_state(CheckpointState::Completed);
        telemetry.record_state(CheckpointState::Rejected);
        telemetry.record_artifact();
        telemetry.record_heartbeat();

        let snapshot = telemetry.snapshot();
        assert_eq!(snapshot.admitted, 2);
        assert_eq!(snapshot.completed, 1);
        assert_eq!(snapshot.rejected, 1);
        assert_eq!(snapshot.failed, 0);
        assert_eq!(snapshot.artifacts_uploaded, 1);
        assert_eq!(snapshot.heartbeats, 1);
    }
}
