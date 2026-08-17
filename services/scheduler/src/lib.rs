//! Local, durable scheduling for the worker's typed `JobEnvelope` values.
//!
//! The scheduler owns queue state, placement, lease fencing, expiry recovery,
//! cancellation, and dead-letter transitions.  It deliberately has no broker,
//! RPC listener, or payload codec: executable `JobEnvelope` values remain in
//! process and lifecycle checkpoints are persisted through an `ObjectStore`.
//!
//! The optional [`http`] module adds a dependency-free HTTP/1.1 transport over
//! the scheduler's SERIALIZABLE CONTROL PLANE only — health, pool registration,
//! job state, checkpoints, cancellation, and expiry recovery. It never dispatches
//! executable work: lease / heartbeat / complete / fail carry or fence typed
//! `JobEnvelope` values that have no wire codec, so they stay in-process.

mod config;
mod error;
mod http;
mod http_error;
mod json;
mod lease;
mod router;
mod scheduler;

pub use config::SchedulerConfig;
pub use error::SchedulerError;
pub use http::{Clock, HttpRequest, HttpResponse, SchedulerServer};
pub use http_error::{TransportError, classify};
pub use lease::{Lease, LeaseToken, SchedulerTransport, WorkerPool};
pub use scheduler::{JobState, PersistedCheckpoint, Scheduler};
