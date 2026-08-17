//! Local-first artifact lifecycle management.
//!
//! The core implements a local storage service; the optional [`http`] module
//! adds a dependency-free HTTP/1.1 transport over that core. Network routing is
//! an explicit, self-contained layer rather than an ambient capability: the core
//! remains usable as a library without binding any listener.

mod authorization;
mod cache;
mod checksum;
mod config;
mod database;
mod download;
mod errors;
mod gc;
mod health;
mod http;
mod http_error;
mod json;
mod limits;
mod metadata;
mod multipart;
mod object;
mod retention;
mod router;
mod routes;
mod signature;
mod storage;
mod telemetry;
mod upload;

pub use http::{ArtifactServer, Clock, HttpRequest, HttpResponse};
pub use http_error::{TransportError, classify};

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
