use lawsynth_worker::JobEnvelope;

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

/// Dispatch of executable work is in-process only. The available forms are
/// surfaced explicitly so callers cannot mistake absence for success.
///
/// `HttpControlPlane` is available, but only for the SERIALIZABLE control plane
/// (health, pools, job state, checkpoints, cancel, recover). It never carries an
/// executable `JobEnvelope` over the wire — that payload has no codec — so typed
/// dispatch (`LocalTyped`) remains the sole way to hand work to a worker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerTransport {
    LocalTyped,
    HttpControlPlane,
    BrokerNotLinked,
    NetworkNotLinked,
}

impl SchedulerTransport {
    pub const fn is_available(self) -> bool {
        matches!(self, Self::LocalTyped | Self::HttpControlPlane)
    }
    pub const fn reason(self) -> &'static str {
        match self {
            Self::LocalTyped => "in-process typed dispatch",
            Self::HttpControlPlane => {
                "HTTP serves the serializable control plane only; executable job dispatch stays in-process"
            }
            Self::BrokerNotLinked => "no broker client or worker-job codec is linked",
            Self::NetworkNotLinked => "no HTTP, RPC, or authentication transport is linked",
        }
    }
}
