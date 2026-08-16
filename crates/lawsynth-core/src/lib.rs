//! Foundational deterministic primitives shared by the LawSynth engine.
//!
//! The public identifiers deliberately reject ambiguous names. World files and
//! expression symbols use these identifiers as their stable, portable handles.

mod cancel;
mod config;
mod diagnostics;
mod error;
mod hash;
mod id;
mod progress;
mod resource;
mod seed;
mod version;

pub use cancel::CancellationToken;
pub use config::EngineConfig;
pub use diagnostics::{Diagnostic, DiagnosticSeverity, Diagnostics};
pub use error::IdentifierError;
pub use hash::stable_hash;
pub use id::Identifier;
pub use progress::{ProgressError, ProgressEvent, ProgressStage, ProgressTracker};
pub use resource::{ResourceLimitError, ResourceLimits};
pub use seed::{DeterministicRng, Seed};
pub use version::{CURRENT_ENGINE_VERSION, EngineVersion, VersionParseError};
