//! Graceful drain coordination for the HTTP server and in-flight leases.
//!
//! A clean shutdown stops admitting new work and then waits for outstanding
//! leases to finish (or a deadline) before exiting, so the scheduler is left in a
//! consistent, reconstructable state. [`Shutdown`] is the shared, thread-safe
//! flag connection threads observe; [`poll_drain`] is the pure step that decides,
//! given the current in-flight count and the clock, whether draining is complete,
//! still in progress, or has timed out. Keeping the decision pure makes the drain
//! loop deterministic and testable without real sleeps or sockets.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// A shared shutdown flag toggled once and observed by many threads.
#[derive(Debug, Default)]
pub struct Shutdown {
    requested: AtomicBool,
}

impl Shutdown {
    /// A fresh, not-yet-requested shutdown handle.
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Signals that shutdown has begun; idempotent.
    pub fn request(&self) {
        self.requested.store(true, Ordering::SeqCst);
    }

    /// Whether shutdown has been requested.
    pub fn is_requested(&self) -> bool {
        self.requested.load(Ordering::SeqCst)
    }
}

/// The state of an in-progress drain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrainStatus {
    /// All in-flight work finished.
    Drained,
    /// Work remains and the deadline has not passed.
    Draining { in_flight: usize },
    /// The deadline passed with work still outstanding.
    TimedOut { in_flight: usize },
}

/// Decides the drain state for the current instant.
///
/// A zero in-flight count is [`DrainStatus::Drained`] regardless of the clock;
/// otherwise the deadline decides between [`DrainStatus::Draining`] and
/// [`DrainStatus::TimedOut`].
pub fn poll_drain(in_flight: usize, now_ms: u64, deadline_ms: u64) -> DrainStatus {
    if in_flight == 0 {
        DrainStatus::Drained
    } else if now_ms >= deadline_ms {
        DrainStatus::TimedOut { in_flight }
    } else {
        DrainStatus::Draining { in_flight }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_starts_clear_and_latches_on_request() {
        let shutdown = Shutdown::new();
        assert!(!shutdown.is_requested());
        shutdown.request();
        shutdown.request();
        assert!(shutdown.is_requested());
    }

    #[test]
    fn drained_when_no_work_remains() {
        assert_eq!(poll_drain(0, 100, 50), DrainStatus::Drained);
    }

    #[test]
    fn draining_before_the_deadline() {
        assert_eq!(poll_drain(2, 10, 50), DrainStatus::Draining { in_flight: 2 });
    }

    #[test]
    fn times_out_at_the_deadline_with_work_left() {
        assert_eq!(poll_drain(1, 50, 50), DrainStatus::TimedOut { in_flight: 1 });
    }

    #[test]
    fn simulated_drain_loop_terminates() {
        let shutdown = Shutdown::new();
        shutdown.request();
        assert!(shutdown.is_requested());
        // A worker count that drains one unit per tick reaches Drained.
        let mut in_flight = 3usize;
        let mut now = 0u64;
        loop {
            match poll_drain(in_flight, now, 100) {
                DrainStatus::Drained => break,
                DrainStatus::Draining { .. } => {
                    in_flight -= 1;
                    now += 10;
                }
                DrainStatus::TimedOut { .. } => panic!("should drain before deadline"),
            }
        }
        assert_eq!(in_flight, 0);
    }
}
