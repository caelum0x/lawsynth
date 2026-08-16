//! A small synchronous execution substrate with explicit admission, cancellation,
//! heartbeats, and verifiable checkpoints.
//!
//! It deliberately does not spawn OS processes or claim sandboxing. Callers
//! supply a cooperative [`WorkProcess`] and own isolation policy.

mod cancellation;
mod checkpoint;
mod config;
mod envelope;
mod error;
mod heartbeat;
mod limits;
mod process;
mod resources;
mod run;

pub use cancellation::CancellationToken;
pub use checkpoint::Checkpoint;
pub use config::RunnerConfig;
pub use envelope::WorkEnvelope;
pub use error::RunnerError;
pub use heartbeat::Heartbeat;
pub use limits::ResourceLimiter;
pub use process::{ProcessRecord, WorkProcess};
pub use resources::ResourceRequest;
pub use run::{ExecutionReport, ExecutionStatus, classify_result, execute};
