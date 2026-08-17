//! A local, synchronous worker for executing LawSynth's typed discovery and
//! simulation operations. It owns admission control and durable lifecycle
//! checkpoints; remote queueing is deliberately separate.
//!
//! The core is usable as a library without binding any listener. The optional
//! [`http`] module adds a dependency-free HTTP/1.1 status transport over that
//! core. That transport is read-only observability -- readiness, admission and
//! config limits, and durable job checkpoints -- and never accepts executable
//! jobs, because [`JobEnvelope`] carries typed, in-memory payloads with no wire
//! codec.

mod checkpoint;
mod config;
mod error;
mod http;
mod http_error;
mod job;
mod json;
mod router;
mod worker;

pub use checkpoint::{CheckpointState, JobCheckpoint};
pub use config::WorkerConfig;
pub use error::WorkerError;
pub use http::{Clock, HttpRequest, HttpResponse, WorkerServer};
pub use http_error::{TransportError, classify};
pub use job::{Job, JobEnvelope, JobOutput, TransportSurface};
pub use worker::{AdmissionSnapshot, Worker};
