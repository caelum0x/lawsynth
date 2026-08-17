//! Readiness and health snapshot.
//!
//! The HTTP `/health` route reports the scheduler's liveness, its live queue
//! depth, the configured bounds, and the lifecycle counters. This module owns the
//! immutable [`HealthSnapshot`] the scheduler produces so the transport layer
//! only renders it — the "what to report" policy lives here, the "how to encode"
//! stays in the router. Readiness is a genuine signal: the scheduler is ready
//! while its live queue has not hit the configured ceiling.

use crate::metrics::MetricsSnapshot;

/// A point-in-time health and readiness view of the scheduler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HealthSnapshot {
    pub ready: bool,
    pub queued_count: usize,
    pub maximum_queued_jobs: usize,
    pub maximum_attempts: u32,
    pub lease_duration_ms: u64,
    pub maximum_checkpoint_bytes: usize,
    pub metrics: MetricsSnapshot,
}

impl HealthSnapshot {
    /// The service identifier reported on the health surface.
    pub const SERVICE: &'static str = "lawsynth-scheduler";

    /// Builds a snapshot, deriving readiness from live queue depth vs. capacity.
    pub fn new(
        queued_count: usize,
        maximum_queued_jobs: usize,
        maximum_attempts: u32,
        lease_duration_ms: u64,
        maximum_checkpoint_bytes: usize,
        metrics: MetricsSnapshot,
    ) -> Self {
        Self {
            ready: queued_count < maximum_queued_jobs,
            queued_count,
            maximum_queued_jobs,
            maximum_attempts,
            lease_duration_ms,
            maximum_checkpoint_bytes,
            metrics,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_ready_with_spare_queue_capacity() {
        let snapshot = HealthSnapshot::new(3, 8, 2, 50, 8192, MetricsSnapshot::default());
        assert!(snapshot.ready);
        assert_eq!(snapshot.queued_count, 3);
    }

    #[test]
    fn is_not_ready_when_queue_is_full() {
        let snapshot = HealthSnapshot::new(8, 8, 2, 50, 8192, MetricsSnapshot::default());
        assert!(!snapshot.ready);
    }
}
