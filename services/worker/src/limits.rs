//! Admission and per-job resource limits.
//!
//! [`crate::WorkerConfig`] carries the pool capacity that the shared admission
//! budget enforces. This module lifts that into an explicit [`Limits`] policy
//! and adds two *optional*, honest per-job ceilings on top of it: a maximum
//! resource request and a maximum wall-clock budget. Both default to absent, so
//! a worker built from a plain config admits exactly what the pool capacity
//! allows -- the shared limiter remains the effective bound. When a ceiling is
//! configured, [`crate::sandbox`] enforces it before a job is admitted.

use lawsynth_runner::ResourceRequest;

use crate::{WorkerConfig, WorkerError};

/// The worker's admission policy: the pool capacity plus optional per-job caps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// The shared pool capacity a worker admits against.
    pub capacity: ResourceRequest,
    /// An optional tighter ceiling on a single job's resource request.
    pub max_job_resources: Option<ResourceRequest>,
    /// An optional ceiling on a single job's wall-clock budget, in milliseconds.
    pub max_wall_ms: Option<u64>,
}

impl Limits {
    /// Derives the baseline policy from a worker configuration. Only the pool
    /// capacity is bounded; the optional per-job ceilings are left absent so
    /// behaviour matches admission against the shared budget alone.
    pub fn from_config(config: &WorkerConfig) -> Self {
        Self { capacity: config.capacity, max_job_resources: None, max_wall_ms: None }
    }

    /// Sets an additional per-job resource ceiling. The ceiling must itself fit
    /// within the pool capacity, otherwise it could never reject anything.
    pub fn with_max_job_resources(mut self, ceiling: ResourceRequest) -> Result<Self, WorkerError> {
        if !ceiling.fits_within(self.capacity) {
            return Err(WorkerError::InvalidConfig(
                "per-job resource ceiling cannot exceed pool capacity".into(),
            ));
        }
        self.max_job_resources = Some(ceiling);
        Ok(self)
    }

    /// Sets an additional per-job wall-clock ceiling in milliseconds.
    pub fn with_max_wall_ms(mut self, max_wall_ms: u64) -> Result<Self, WorkerError> {
        if max_wall_ms == 0 {
            return Err(WorkerError::InvalidConfig(
                "per-job wall-clock ceiling must be positive".into(),
            ));
        }
        self.max_wall_ms = Some(max_wall_ms);
        Ok(self)
    }

    /// Checks a job's request and wall-clock budget against the configured
    /// ceilings. The pool-capacity admission is enforced separately by the
    /// shared limiter; this method reports only the additional per-job caps.
    pub fn admits(&self, request: ResourceRequest, wall_ms: u64) -> Result<(), WorkerError> {
        if let Some(ceiling) = self.max_job_resources {
            if !request.fits_within(ceiling) {
                return Err(WorkerError::LimitExceeded(format!(
                    "job requests {} cpu-millis / {} memory bytes, ceiling is {} / {}",
                    request.cpu_millis,
                    request.memory_bytes,
                    ceiling.cpu_millis,
                    ceiling.memory_bytes
                )));
            }
        }
        if let Some(max_wall_ms) = self.max_wall_ms {
            if wall_ms > max_wall_ms {
                return Err(WorkerError::LimitExceeded(format!(
                    "job wall-clock budget {wall_ms} ms exceeds ceiling {max_wall_ms} ms"
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capacity() -> ResourceRequest {
        ResourceRequest::new(1_000, 1 << 20, 1 << 20).unwrap()
    }

    #[test]
    fn baseline_policy_admits_anything_within_capacity() {
        let config = WorkerConfig::new(capacity(), 1 << 10).unwrap();
        let limits = Limits::from_config(&config);
        assert!(limits.max_job_resources.is_none());
        let request = ResourceRequest::new(999, 1 << 19, 1 << 19).unwrap();
        assert!(limits.admits(request, u64::MAX).is_ok());
    }

    #[test]
    fn configured_ceilings_reject_oversized_requests_and_budgets() {
        let config = WorkerConfig::new(capacity(), 1 << 10).unwrap();
        let ceiling = ResourceRequest::new(500, 1 << 19, 1 << 19).unwrap();
        let limits = Limits::from_config(&config)
            .with_max_job_resources(ceiling)
            .unwrap()
            .with_max_wall_ms(1_000)
            .unwrap();

        let too_much_cpu = ResourceRequest::new(600, 1 << 19, 1 << 19).unwrap();
        assert!(matches!(limits.admits(too_much_cpu, 500), Err(WorkerError::LimitExceeded(_))));

        let fits = ResourceRequest::new(400, 1 << 18, 1 << 18).unwrap();
        assert!(limits.admits(fits, 500).is_ok());
        assert!(matches!(limits.admits(fits, 2_000), Err(WorkerError::LimitExceeded(_))));
    }

    #[test]
    fn a_ceiling_above_capacity_is_rejected() {
        let config = WorkerConfig::new(capacity(), 1 << 10).unwrap();
        let over = ResourceRequest::new(2_000, 1 << 20, 1 << 20).unwrap();
        assert!(matches!(
            Limits::from_config(&config).with_max_job_resources(over),
            Err(WorkerError::InvalidConfig(_))
        ));
    }
}
