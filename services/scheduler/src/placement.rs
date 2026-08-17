//! Placement predicate: can a queued job run on a pool right now?
//!
//! Placement is the admission gate applied to every queued job during
//! [`crate::Scheduler::lease_next`]: a job is placeable only if it has not
//! already passed its deadline and its resource request fits within the pool's
//! currently-available capacity. Extracting this predicate keeps the selection
//! loop declarative and makes the placement rule independently testable.

use lawsynth_runner::ResourceRequest;

/// Whether a queued job can be assigned to a pool with `available` resources.
///
/// An expired job is never placed (it will be dead-lettered instead), and a job
/// that does not fit the free capacity is skipped in favor of a smaller one.
pub fn is_placeable(request: ResourceRequest, available: ResourceRequest, expired: bool) -> bool {
    !expired && request.fits_within(available)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(cpu: u32, mem: u64, disk: u64) -> ResourceRequest {
        ResourceRequest::new(cpu, mem, disk).unwrap()
    }

    #[test]
    fn places_a_fitting_live_job() {
        assert!(is_placeable(r(250, 1024, 1024), r(500, 4096, 4096), false));
    }

    #[test]
    fn refuses_an_expired_job_even_when_it_fits() {
        assert!(!is_placeable(r(250, 1024, 1024), r(500, 4096, 4096), true));
    }

    #[test]
    fn refuses_a_job_that_exceeds_available_capacity() {
        assert!(!is_placeable(r(600, 1024, 1024), r(500, 4096, 4096), false));
    }
}
