use lawsynth_runner::ResourceRequest;
use lawsynth_worker::JobEnvelope;

use crate::SchedulerError;

/// A named, resource-bounded worker pool available to the local scheduler.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerPool {
    pub id: String,
    pub capacity: ResourceRequest,
}

impl WorkerPool {
    pub fn new(id: impl Into<String>, capacity: ResourceRequest) -> Result<Self, SchedulerError> {
        let id = id.into();
        if id.is_empty()
            || id.len() > 128
            || !id.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return Err(SchedulerError::InvalidWorker(
                "id must be URL-safe and no longer than 128 bytes".into(),
            ));
        }
        Ok(Self { id, capacity })
    }
}

/// Fencing token carried by a worker. A later lease for the same job always
/// has a higher generation, so old workers cannot complete it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaseToken {
    pub job_id: String,
    pub worker_id: String,
    pub generation: u64,
}

/// An assigned executable worker envelope and its bounded lease.
#[derive(Clone, Debug, PartialEq)]
pub struct Lease {
    pub token: LeaseToken,
    pub issued_at_ms: u64,
    pub expires_at_ms: u64,
    pub envelope: JobEnvelope,
}

/// This crate supports only in-process dispatch. Queue/broker and network
/// forms are surfaced explicitly so callers cannot mistake absence for success.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerTransport {
    LocalTyped,
    BrokerNotLinked,
    NetworkNotLinked,
}

impl SchedulerTransport {
    pub const fn is_available(self) -> bool {
        matches!(self, Self::LocalTyped)
    }
    pub const fn reason(self) -> &'static str {
        match self {
            Self::LocalTyped => "in-process typed dispatch",
            Self::BrokerNotLinked => "no broker client or worker-job codec is linked",
            Self::NetworkNotLinked => "no HTTP, RPC, or authentication transport is linked",
        }
    }
}
