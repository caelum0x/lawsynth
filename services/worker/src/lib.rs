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
//!
//! The worker is decomposed into focused modules: [`execute`] runs the typed
//! job; [`resources`] and [`limits`] account and bound admission; [`sandbox`]
//! enforces the deadline and configured resource ceilings honestly (documenting
//! what an OS sandbox would add); [`heartbeat`] and [`lease`] model the worker's
//! side of a scheduler lease with fencing; [`events`] and [`telemetry`] record
//! lifecycle observability; [`recovery`], [`cleanup`], [`artifacts`], and
//! [`upload`] handle interrupted-job recovery, scratch reclamation, and the
//! checksum-verified artifact handoff; [`shutdown`] coordinates graceful drain;
//! and [`plugins`] is an honest, un-faked seam onto a plugin runtime that this
//! build does not link.

mod artifacts;
mod checkpoint;
mod cleanup;
mod config;
mod errors;
mod events;
mod execute;
mod health;
mod heartbeat;
mod http;
mod http_error;
mod job;
mod json;
mod lease;
mod limits;
mod plugins;
mod recovery;
mod resources;
mod router;
mod sandbox;
mod shutdown;
mod telemetry;
mod upload;
mod worker;

pub use artifacts::{ArtifactManifest, ArtifactReceipt};
pub use checkpoint::{CheckpointState, JobCheckpoint};
pub use cleanup::{CleanupReport, scratch_key};
pub use config::WorkerConfig;
pub use errors::WorkerError;
pub use events::{EventLog, JobEvent};
pub use health::HealthSnapshot;
pub use heartbeat::{Heartbeat, HeartbeatState};
pub use http::{Clock, HttpRequest, HttpResponse, WorkerServer};
pub use http_error::{TransportError, classify};
pub use job::{Job, JobEnvelope, JobOutput, TransportSurface};
pub use lease::{LeaseState, LeaseToken, WorkerLease};
pub use limits::Limits;
pub use plugins::{PluginDispatch, PluginKind, PluginOutcome, PluginRequest, PluginSeam};
pub use recovery::{RecoveryAction, RecoveryPlan};
pub use resources::AdmissionSnapshot;
pub use sandbox::Sandbox;
pub use shutdown::{DrainState, ShutdownController, WorkGuard};
pub use telemetry::{Telemetry, TelemetrySnapshot};
pub use upload::UploadReceipt;
pub use worker::Worker;
