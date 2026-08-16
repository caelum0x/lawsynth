//! Deterministic, inspectable bundle I/O for initial continuous World IR.

mod canonical;
mod checksum;
mod config;
mod error;
mod layout;
mod manifest;
mod migration;
mod reader;
mod signature;
mod writer;

pub use canonical::canonical_entry_order;
pub use checksum::sha256_hex;
pub use config::BundleConfig;
pub use error::BundleError;
pub use migration::{BundleFormatVersion, migration_path};
pub use reader::{read_discrete_world, read_world};
pub use signature::{BundleSignature, verify_signature};
pub use writer::{write_discrete_world, write_world};
