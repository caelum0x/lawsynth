//! The worker's readiness snapshot.
//!
//! This is the worker-state half of what `GET /health` serves: whether the
//! worker is ready, its live admission budget, and its checkpoint size bound.
//! The transport-specific fields of the health response (the request timestamp
//! and the transport-surface description) stay in [`crate::router`]; this type
//! is the portable, socket-free view that the router renders and that callers
//! can obtain directly via [`crate::Worker::health`].

use crate::AdmissionSnapshot;

/// A consistent readiness view of a worker at a single instant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HealthSnapshot {
    /// The worker is ready whenever it is running; capacity pressure is reported
    /// through [`HealthSnapshot::admission`], not by flipping readiness.
    pub ready: bool,
    /// The live capacity/reserved/available admission triple.
    pub admission: AdmissionSnapshot,
    /// The maximum size, in bytes, of a persisted lifecycle checkpoint.
    pub maximum_checkpoint_bytes: usize,
}

impl HealthSnapshot {
    pub(crate) fn new(admission: AdmissionSnapshot, maximum_checkpoint_bytes: usize) -> Self {
        Self { ready: true, admission, maximum_checkpoint_bytes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lawsynth_runner::ResourceRequest;

    #[test]
    fn snapshot_is_ready_and_carries_the_admission_triple() {
        let capacity = ResourceRequest::new(1_000, 1 << 20, 1 << 20).unwrap();
        let reserved = ResourceRequest::new(1, 1, 0).unwrap();
        let available = capacity.checked_sub(reserved).unwrap();
        let admission = AdmissionSnapshot { capacity, reserved, available };
        let snapshot = HealthSnapshot::new(admission, 4096);
        assert!(snapshot.ready);
        assert_eq!(snapshot.admission.capacity, capacity);
        assert_eq!(snapshot.maximum_checkpoint_bytes, 4096);
    }
}
