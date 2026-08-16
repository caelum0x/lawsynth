//! Local-first artifact lifecycle management.
//!
//! This crate deliberately implements a local storage service, not an HTTP server.
//! Its public API is usable by a daemon or CLI and makes the availability boundary
//! explicit: network routing, remote identity, and distributed metadata are not
//! linked into this crate.

mod authorization;
mod cache;
mod checksum;
mod config;
mod database;
mod download;
mod errors;
mod gc;
mod health;
mod limits;
mod metadata;
mod multipart;
mod object;
mod retention;
mod routes;
mod signature;
mod storage;
mod telemetry;
mod upload;

pub use authorization::{AccessAction, LocalOnlyAuthorizer};
pub use checksum::{is_sha256_hex, sha256};
pub use config::ArtifactConfig;
pub use database::ArtifactCatalog;
pub use errors::ArtifactError;
pub use gc::GarbageCollectionReport;
pub use health::HealthReport;
pub use metadata::ArtifactMetadata;
pub use multipart::UploadId;
pub use object::{Artifact, ArtifactId, UploadOptions};
pub use retention::Retention;
pub use routes::{LocalOperation, NetworkSurface};
pub use signature::{BundleAuthenticator, BundleVerification};
pub use telemetry::{Telemetry, TelemetrySnapshot};
pub use upload::ArtifactService;
