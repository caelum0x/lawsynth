//! Deterministic storage primitives for LawSynth artifacts.
//! `MemoryStore` and `LocalStore` are complete implementations. `S3Store` validates
//! endpoint configuration but intentionally does not embed HTTP/TLS/signing.
mod cache;
mod config;
mod error;
mod gc;
mod local;
mod memory;
mod multipart;
mod object;
mod s3;
mod store;
pub use cache::ObjectCache;
pub use config::StoreConfig;
pub use error::StoreError;
pub use gc::{GcReport, collect_unreferenced};
pub use local::LocalStore;
pub use memory::MemoryStore;
pub use multipart::MultipartUpload;
pub use object::{Object, ObjectKey, checksum};
pub use s3::{S3Config, S3Store};
pub use store::ObjectStore;
