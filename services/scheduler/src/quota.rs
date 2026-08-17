//! Per-pool resource quota arithmetic.
//!
//! This module owns the pure, checked resource math the scheduler uses to admit
//! work into a worker pool: how much capacity remains, how a reservation grows,
//! and how a completion releases it. It holds no state — [`crate::pool`] threads
//! the running `reserved` total through these functions — so the accounting is
//! trivially testable and cannot silently overflow or go negative.

use lawsynth_runner::ResourceRequest;

use crate::SchedulerError;

/// The additive identity for a resource vector: an empty reservation.
pub const fn zero() -> ResourceRequest {
    ResourceRequest { cpu_millis: 0, memory_bytes: 0, disk_bytes: 0 }
}

/// Resources still free in a pool, i.e. `capacity - reserved`.
///
/// A reservation exceeding capacity is a corrupt invariant rather than a caller
/// error, so it surfaces as [`SchedulerError::CorruptCheckpoint`].
pub fn available(
    capacity: ResourceRequest,
    reserved: ResourceRequest,
    pool_id: &str,
) -> Result<ResourceRequest, SchedulerError> {
    capacity.checked_sub(reserved).ok_or_else(|| {
        SchedulerError::CorruptCheckpoint(format!("pool '{pool_id}' reserved beyond capacity"))
    })
}

/// The new reserved total after admitting `request`, refusing over-commitment.
///
/// Overflow of the running total is a corrupt invariant; a request that would
/// exceed the pool's capacity is an [`SchedulerError::InvalidWorker`] admission
/// failure the caller can react to.
pub fn reserve(
    reserved: ResourceRequest,
    capacity: ResourceRequest,
    request: ResourceRequest,
    pool_id: &str,
) -> Result<ResourceRequest, SchedulerError> {
    let next = reserved
        .checked_add(request)
        .ok_or_else(|| SchedulerError::CorruptCheckpoint("resource reservation overflow".into()))?;
    if !next.fits_within(capacity) {
        return Err(SchedulerError::InvalidWorker(format!(
            "pool '{pool_id}' cannot reserve assigned job"
        )));
    }
    Ok(next)
}

/// The new reserved total after releasing `request`.
///
/// Releasing more than was reserved is a corrupt invariant.
pub fn release(
    reserved: ResourceRequest,
    request: ResourceRequest,
    pool_id: &str,
) -> Result<ResourceRequest, SchedulerError> {
    reserved.checked_sub(request).ok_or_else(|| {
        SchedulerError::CorruptCheckpoint(format!("pool '{pool_id}' released unreserved resources"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(cpu: u32, mem: u64, disk: u64) -> ResourceRequest {
        ResourceRequest { cpu_millis: cpu, memory_bytes: mem, disk_bytes: disk }
    }

    #[test]
    fn available_is_capacity_minus_reserved() {
        let free = available(r(500, 4096, 4096), r(200, 1024, 0), "cpu-a").unwrap();
        assert_eq!(free, r(300, 3072, 4096));
    }

    #[test]
    fn available_rejects_reservation_beyond_capacity() {
        let error = available(r(100, 1024, 1024), r(200, 1024, 1024), "cpu-a").unwrap_err();
        assert!(matches!(error, SchedulerError::CorruptCheckpoint(_)));
    }

    #[test]
    fn reserve_accumulates_within_capacity() {
        let next = reserve(zero(), r(500, 4096, 4096), r(250, 1024, 1024), "cpu-a").unwrap();
        assert_eq!(next, r(250, 1024, 1024));
    }

    #[test]
    fn reserve_refuses_over_commitment_as_invalid_worker() {
        let error = reserve(r(400, 0, 0), r(500, 4096, 4096), r(200, 0, 0), "cpu-a").unwrap_err();
        assert!(matches!(error, SchedulerError::InvalidWorker(_)));
    }

    #[test]
    fn release_underflow_is_corrupt() {
        let error = release(r(100, 0, 0), r(200, 0, 0), "cpu-a").unwrap_err();
        assert!(matches!(error, SchedulerError::CorruptCheckpoint(_)));
    }
}
