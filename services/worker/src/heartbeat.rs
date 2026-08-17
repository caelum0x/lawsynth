//! A deterministic lease-heartbeat state machine.
//!
//! A worker holding a lease must periodically prove liveness to the scheduler;
//! if it stops, the lease expires and the job returns to a schedulable state
//! (production architecture, sections 10 and 23). This models the worker's side
//! of that contract as a pure state machine driven by an injected millisecond
//! clock, so the "is a beat due?" and "has the lease lapsed?" decisions are
//! reproducible in tests with no wall-clock dependence.

use crate::WorkerError;

/// The heartbeat's status relative to the current instant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeartbeatState {
    /// A beat was sent recently; nothing to do yet.
    Fresh,
    /// The interval has elapsed; the worker should beat now.
    Due,
    /// The lease TTL elapsed without a beat; the scheduler will reclaim the job.
    Expired,
}

/// Periodic heartbeat state for one held lease.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Heartbeat {
    interval_ms: u64,
    lease_ttl_ms: u64,
    last_beat_ms: u64,
    sequence: u64,
}

impl Heartbeat {
    /// Starts a heartbeat at `now_ms`, counting the start as the first observed
    /// beat (sequence one). The interval must be positive and the lease TTL at
    /// least one interval, otherwise the lease could lapse before the first
    /// scheduled beat.
    pub fn start(now_ms: u64, interval_ms: u64, lease_ttl_ms: u64) -> Result<Self, WorkerError> {
        if interval_ms == 0 {
            return Err(WorkerError::InvalidConfig("heartbeat interval must be positive".into()));
        }
        if lease_ttl_ms < interval_ms {
            return Err(WorkerError::InvalidConfig(
                "lease TTL must be at least one heartbeat interval".into(),
            ));
        }
        Ok(Self { interval_ms, lease_ttl_ms, last_beat_ms: now_ms, sequence: 1 })
    }

    /// The sequence number of the most recent beat.
    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    /// The instant of the most recent beat.
    pub fn last_beat_ms(&self) -> u64 {
        self.last_beat_ms
    }

    /// The instant at which the lease lapses if no further beat is recorded.
    pub fn expires_at_ms(&self) -> u64 {
        self.last_beat_ms.saturating_add(self.lease_ttl_ms)
    }

    /// Whether a beat is due at `now_ms` (a full interval since the last beat).
    pub fn is_due(&self, now_ms: u64) -> bool {
        now_ms.saturating_sub(self.last_beat_ms) >= self.interval_ms
    }

    /// Whether the lease has lapsed at `now_ms`.
    pub fn is_expired(&self, now_ms: u64) -> bool {
        now_ms.saturating_sub(self.last_beat_ms) > self.lease_ttl_ms
    }

    /// Classifies the heartbeat at `now_ms`. Expiry takes precedence over a due
    /// beat: once the lease has lapsed, beating no longer reclaims it.
    pub fn state(&self, now_ms: u64) -> HeartbeatState {
        if self.is_expired(now_ms) {
            HeartbeatState::Expired
        } else if self.is_due(now_ms) {
            HeartbeatState::Due
        } else {
            HeartbeatState::Fresh
        }
    }

    /// Records a beat at `now_ms`, advancing the sequence and refreshing the
    /// lease. Beating after the lease has lapsed is refused: the worker must
    /// acquire a fresh lease rather than silently resurrect an expired one.
    pub fn beat(&mut self, now_ms: u64) -> Result<u64, WorkerError> {
        if self.is_expired(now_ms) {
            return Err(WorkerError::LimitExceeded(
                "lease TTL elapsed before heartbeat; a fresh lease is required".into(),
            ));
        }
        self.sequence += 1;
        self.last_beat_ms = now_ms;
        Ok(self.sequence)
    }
}
