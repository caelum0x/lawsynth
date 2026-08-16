//! A local, synchronous worker for executing LawSynth's typed discovery and
//! simulation operations. It owns admission control and durable lifecycle
//! checkpoints; transport and remote queueing are deliberately separate.

mod checkpoint;
mod config;
mod error;
mod job;
mod worker;

pub use checkpoint::{CheckpointState, JobCheckpoint};
pub use config::WorkerConfig;
pub use error::WorkerError;
pub use job::{Job, JobEnvelope, JobOutput, TransportSurface};
pub use worker::Worker;
