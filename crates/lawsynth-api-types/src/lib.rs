//! Transport-neutral, validated values used by LawSynth HTTP, CLI, and job APIs.
//!
//! This crate contains no server implementation and intentionally performs no
//! serialization. It establishes the invariants that transport adapters must
//! preserve before work is admitted to a runner.

mod artifact;
mod config;
mod dataset;
mod error;
mod events;
mod pagination;
mod project;
mod run;
mod simulation;
mod world;

pub use artifact::{ArtifactDescriptor, ArtifactId, ArtifactMediaType};
pub use config::ApiLimits;
pub use dataset::{ColumnType, DatasetColumn, DatasetDescriptor, DatasetId};
pub use error::ApiValidationError;
pub use events::{ApiEvent, EventKind, validate_event_stream};
pub use pagination::{Page, PageRequest};
pub use project::{Project, ProjectId};
pub use run::{RunId, RunStatus, RunSummary};
pub use simulation::{SimulationRequest, TimeRange};
pub use world::{WorldId, WorldRevision};
