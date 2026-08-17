//! Local, durable scheduling for the worker's typed `JobEnvelope` values.
//!
//! The scheduler owns queue state, placement, lease fencing, expiry recovery,
//! cancellation, and dead-letter transitions.  It deliberately has no broker,
//! RPC listener, or payload codec: executable `JobEnvelope` values remain in
//! process and lifecycle checkpoints are persisted through an `ObjectStore`.
//!
//! The core is decomposed into focused modules, each independently testable:
//! [`queue`] and [`priority`] order and select queued work; [`placement`] gates
//! admission; [`pool`] and [`quota`] account for per-pool resources; [`policy`],
//! [`backoff`], and [`recovery`] decide retry, delay, and dead-letter; [`database`]
//! persists checkpoints; [`metrics`] and [`events`] expose lifecycle observability;
//! [`health`] and [`shutdown`] support ops; [`fairness`] spreads multi-pool
//! dispatch; and [`nats`] is an honest broker seam that links no client.
//!
//! The optional [`http`] module adds a dependency-free HTTP/1.1 transport over
//! the scheduler's SERIALIZABLE CONTROL PLANE only — health, pool registration,
//! job state, checkpoints, cancellation, and expiry recovery. It never dispatches
//! executable work: lease / heartbeat / complete / fail carry or fence typed
//! `JobEnvelope` values that have no wire codec, so they stay in-process.

mod backoff;
mod config;
mod database;
mod error;
mod errors;
mod events;
mod fairness;
mod health;
mod http;
mod http_error;
mod json;
mod lease;
mod metrics;
mod nats;
mod placement;
mod policy;
mod pool;
mod priority;
mod queue;
mod quota;
mod recovery;
mod router;
mod scheduler;
mod shutdown;

pub use backoff::Backoff;
pub use config::SchedulerConfig;
pub use database::CheckpointStore;
pub use error::SchedulerError;
pub use errors::SchedulerResult;
pub use events::{EventLog, EventRecord, JobEvent};
pub use fairness::FairShare;
pub use health::HealthSnapshot;
pub use http::{Clock, HttpRequest, HttpResponse, SchedulerServer};
pub use http_error::{TransportError, classify};
pub use lease::{Lease, LeaseToken, SchedulerTransport};
pub use metrics::{MetricsSnapshot, SchedulerMetrics};
pub use nats::{BrokerError, JobBroker, UnlinkedBroker};
pub use placement::is_placeable;
pub use policy::{FailureAction, RetryPolicy};
pub use pool::{PoolRegistry, WorkerPool};
pub use priority::Candidate;
pub use queue::select as select_next;
pub use recovery::{RecoveryOutcome, on_lease_expiry};
pub use scheduler::{JobState, PersistedCheckpoint, Scheduler};
pub use shutdown::{DrainStatus, Shutdown, poll_drain};
