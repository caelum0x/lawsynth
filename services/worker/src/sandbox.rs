//! A resource-bound guard around job execution.
//!
//! HONESTY BOUNDARY: this is not an OS sandbox. A production deployment isolates
//! a worker with kernel facilities -- cgroup CPU/memory limits, `rlimit` caps, a
//! `seccomp` syscall filter, and mount/network namespaces (see the production
//! architecture, sections 10 and 22). Those are platform-specific and cannot be
//! enforced portably from safe, std-only Rust, so they are *not* claimed here.
//!
//! What this guard enforces honestly and deterministically is the portable
//! contract the worker already owns: the job's wall-clock **deadline** and the
//! declared per-job resource **ceilings** from [`crate::Limits`]. Time is
//! injected, so admission decisions are reproducible. A worker built from a
//! plain config carries no extra ceiling, so this guard reduces to the same
//! deadline check the worker has always performed; configured ceilings add a
//! real, tested layer on top.

use lawsynth_runner::WorkEnvelope;

use crate::{Limits, WorkerError};

/// A deterministic admission guard parameterised by an admission policy.
#[derive(Clone, Copy, Debug)]
pub struct Sandbox {
    limits: Limits,
}

impl Sandbox {
    pub fn new(limits: Limits) -> Self {
        Self { limits }
    }

    /// The policy this sandbox enforces.
    pub fn limits(&self) -> Limits {
        self.limits
    }

    /// Rejects a job whose deadline has already elapsed at `now_ms`.
    pub fn check_deadline(&self, work: &WorkEnvelope, now_ms: u64) -> Result<(), WorkerError> {
        if work.is_expired(now_ms) {
            return Err(WorkerError::DeadlineExceeded {
                job_id: work.id.clone(),
                deadline_at_ms: work.deadline_at_ms,
            });
        }
        Ok(())
    }

    /// Full admission check: the deadline plus any configured per-job ceilings.
    ///
    /// The wall-clock budget measured against [`Limits::max_wall_ms`] is the
    /// span from submission to deadline, which is independent of `now_ms` and so
    /// stays stable across retries of the same envelope.
    pub fn admit(&self, work: &WorkEnvelope, now_ms: u64) -> Result<(), WorkerError> {
        self.check_deadline(work, now_ms)?;
        let wall_ms = work.deadline_at_ms.saturating_sub(work.submitted_at_ms);
        self.limits.admits(work.resources, wall_ms)
    }

    /// Detects a blown deadline after synchronous work returns. The engines are
    /// cooperative and do not accept a deadline themselves, so a caller that
    /// wants hard-timeout accounting samples the clock again and passes the
    /// observed instant here; a job whose deadline elapsed while running is
    /// reported as [`WorkerError::DeadlineExceeded`].
    pub fn check_overrun(
        &self,
        work: &WorkEnvelope,
        finished_at_ms: u64,
    ) -> Result<(), WorkerError> {
        self.check_deadline(work, finished_at_ms)
    }
}
