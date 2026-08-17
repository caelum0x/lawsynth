//! The worker's view of a held lease, with fencing.
//!
//! When a scheduler hands a job to a worker it issues a lease carrying a fencing
//! token: a monotonically increasing generation for that job id. If the lease
//! expires and the job is re-assigned, the new lease has a higher generation, so
//! a stale worker that wakes up late is *fenced* and must not finalize the job
//! (production architecture, sections 10 and 23: "a later lease for the same job
//! always has a higher generation"). This module is the worker-side, injected-
//! clock view of that contract; the scheduler owns issuance.

use crate::WorkerError;

/// A fencing token identifying who currently owns a job and at which generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaseToken {
    pub job_id: String,
    pub worker_id: String,
    pub generation: u64,
}

impl LeaseToken {
    pub fn new(
        job_id: impl Into<String>,
        worker_id: impl Into<String>,
        generation: u64,
    ) -> Result<Self, WorkerError> {
        let job_id = job_id.into();
        let worker_id = worker_id.into();
        if !is_url_safe(&job_id) {
            return Err(WorkerError::InvalidJob(
                "lease job id must be URL-safe and no longer than 128 bytes".into(),
            ));
        }
        if !is_url_safe(&worker_id) {
            return Err(WorkerError::InvalidJob(
                "lease worker id must be URL-safe and no longer than 128 bytes".into(),
            ));
        }
        Ok(Self { job_id, worker_id, generation })
    }

    /// Whether `other` supersedes this token: same job, strictly newer generation.
    pub fn is_superseded_by(&self, other: &LeaseToken) -> bool {
        self.job_id == other.job_id && other.generation > self.generation
    }
}

/// The lease's status relative to the current instant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LeaseState {
    Held,
    Expired,
}

/// A bounded lease a worker holds for one job.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerLease {
    pub token: LeaseToken,
    pub issued_at_ms: u64,
    pub expires_at_ms: u64,
}

impl WorkerLease {
    pub fn new(
        token: LeaseToken,
        issued_at_ms: u64,
        expires_at_ms: u64,
    ) -> Result<Self, WorkerError> {
        if expires_at_ms <= issued_at_ms {
            return Err(WorkerError::InvalidJob("lease expiry must follow its issuance".into()));
        }
        Ok(Self { token, issued_at_ms, expires_at_ms })
    }

    pub fn is_expired(&self, now_ms: u64) -> bool {
        now_ms >= self.expires_at_ms
    }

    pub fn state(&self, now_ms: u64) -> LeaseState {
        if self.is_expired(now_ms) { LeaseState::Expired } else { LeaseState::Held }
    }

    /// Whether this lease may finalize its job given the currently authoritative
    /// token. A stale worker is fenced when a newer generation exists.
    pub fn may_commit(&self, authoritative: &LeaseToken) -> bool {
        self.token.job_id == authoritative.job_id
            && self.token.generation >= authoritative.generation
    }

    /// Extends the lease to a new expiry without changing the fencing token,
    /// modelling a successful heartbeat-driven renewal.
    pub fn renew(&self, now_ms: u64, ttl_ms: u64) -> Result<Self, WorkerError> {
        if self.is_expired(now_ms) {
            return Err(WorkerError::LimitExceeded(
                "cannot renew a lease that has already expired".into(),
            ));
        }
        if ttl_ms == 0 {
            return Err(WorkerError::InvalidConfig("lease renewal TTL must be positive".into()));
        }
        Self::new(self.token.clone(), now_ms, now_ms.saturating_add(ttl_ms))
    }
}

fn is_url_safe(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}
