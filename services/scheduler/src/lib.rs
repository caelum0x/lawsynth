//! Local, durable scheduling for the worker's typed `JobEnvelope` values.
//!
//! The scheduler owns queue state, placement, lease fencing, expiry recovery,
//! cancellation, and dead-letter transitions.  It deliberately has no broker,
//! RPC listener, or payload codec: executable `JobEnvelope` values remain in
//! process and lifecycle checkpoints are persisted through an `ObjectStore`.

mod config;
mod error;
mod lease;
mod scheduler;

pub use config::SchedulerConfig;
pub use error::SchedulerError;
pub use lease::{Lease, LeaseToken, SchedulerTransport, WorkerPool};
pub use scheduler::{JobState, PersistedCheckpoint, Scheduler};
