//! In-process lifecycle counters.
//!
//! The scheduler increments these monotonic counters as jobs move through their
//! lifecycle, giving operators the "runs by state" and "queue" signals the
//! reliability chapter calls for without linking a metrics backend. The counters
//! are cumulative totals (not gauges): `queued` counts every enqueue including
//! retries, so `queued - leased` is not a live depth — the live queue depth is
//! [`crate::Scheduler::queued_count`]. A [`MetricsSnapshot`] is a cheap immutable
//! copy safe to render over the control plane.

/// An immutable point-in-time copy of the scheduler's counters.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MetricsSnapshot {
    pub queued: u64,
    pub leased: u64,
    pub completed: u64,
    pub failed: u64,
    pub cancelled: u64,
    pub dead_letter: u64,
}

/// Mutable cumulative counters owned by the scheduler.
#[derive(Clone, Copy, Debug, Default)]
pub struct SchedulerMetrics {
    snapshot: MetricsSnapshot,
}

impl SchedulerMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// A job entered the queue (initial submission or a retry requeue).
    pub fn record_queued(&mut self) {
        self.snapshot.queued = self.snapshot.queued.saturating_add(1);
    }

    /// A job was leased to a worker.
    pub fn record_leased(&mut self) {
        self.snapshot.leased = self.snapshot.leased.saturating_add(1);
    }

    /// A job completed successfully.
    pub fn record_completed(&mut self) {
        self.snapshot.completed = self.snapshot.completed.saturating_add(1);
    }

    /// A worker reported a failure (before the requeue/dead-letter decision).
    pub fn record_failed(&mut self) {
        self.snapshot.failed = self.snapshot.failed.saturating_add(1);
    }

    /// A job was cancelled by the control plane.
    pub fn record_cancelled(&mut self) {
        self.snapshot.cancelled = self.snapshot.cancelled.saturating_add(1);
    }

    /// A job reached the dead-letter terminal state.
    pub fn record_dead_letter(&mut self) {
        self.snapshot.dead_letter = self.snapshot.dead_letter.saturating_add(1);
    }

    /// A cheap immutable copy of the current counters.
    pub fn snapshot(&self) -> MetricsSnapshot {
        self.snapshot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_start_at_zero() {
        assert_eq!(SchedulerMetrics::new().snapshot(), MetricsSnapshot::default());
    }

    #[test]
    fn records_accumulate_independently() {
        let mut metrics = SchedulerMetrics::new();
        metrics.record_queued();
        metrics.record_queued();
        metrics.record_leased();
        metrics.record_completed();
        metrics.record_failed();
        metrics.record_dead_letter();
        metrics.record_cancelled();
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.queued, 2);
        assert_eq!(snapshot.leased, 1);
        assert_eq!(snapshot.completed, 1);
        assert_eq!(snapshot.failed, 1);
        assert_eq!(snapshot.dead_letter, 1);
        assert_eq!(snapshot.cancelled, 1);
    }
}
