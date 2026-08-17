//! Resource accounting and reservation over the shared admission budget.
//!
//! The worker admits concurrent callers against one explicit capacity budget.
//! This module owns the reservation arithmetic and the consistent read-only
//! [`AdmissionSnapshot`] the status surface reports, keeping that accounting in
//! one place rather than inlined in the execution path.

use lawsynth_runner::{ResourceLimiter, ResourceRequest};

use crate::WorkerError;

/// A consistent view of the worker's admission budget for status reporting.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmissionSnapshot {
    pub capacity: ResourceRequest,
    pub reserved: ResourceRequest,
    pub available: ResourceRequest,
}

/// Captures a coherent capacity/reserved/available triple from a locked limiter.
pub(crate) fn snapshot(limiter: &ResourceLimiter) -> AdmissionSnapshot {
    AdmissionSnapshot {
        capacity: limiter.capacity(),
        reserved: limiter.reserved(),
        available: limiter.available(),
    }
}

/// Reserves a job's resources against the shared budget, mapping a refusal into
/// the worker's error taxonomy.
pub(crate) fn reserve(
    limiter: &mut ResourceLimiter,
    request: ResourceRequest,
) -> Result<(), WorkerError> {
    limiter.reserve(request).map_err(WorkerError::from)
}

/// Returns a completed job's resources to the shared budget.
pub(crate) fn release(
    limiter: &mut ResourceLimiter,
    request: ResourceRequest,
) -> Result<(), WorkerError> {
    limiter.release(request).map_err(WorkerError::from)
}
