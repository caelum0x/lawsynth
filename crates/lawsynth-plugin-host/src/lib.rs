//! Isolated plugin host controls.  This crate never grants filesystem or
//! network access implicitly and uses process boundaries for executable plugins.

mod config;
mod discover;
mod error;
mod install;
mod lifecycle;
mod package;
mod permissions;
mod process;
mod registry;
mod resources;
mod rpc;
mod trust;
mod wasi;

pub use config::HostConfig;
pub use discover::discover_manifests;
pub use error::HostError;
pub use install::{InstalledPlugin, TrustStatus};
pub use lifecycle::HostedPlugin;
pub use package::{
    CHECKSUMS_PATH, MANIFEST_PATH, PackageSignature, PluginPackage, SIGNATURE_PATH,
    build_checksums, pack, package_hash_of, unpack,
};
pub use permissions::{PermissionPolicy, PermissionSet};
pub use process::{ProcessHandle, ProcessSpec};
pub use registry::PluginRegistry;
pub use resources::ResourceMeter;
pub use rpc::{RpcChannel, read_frame, write_frame};
pub use trust::{TrustStore, sign_package_hash, verify_with_secret};
pub use wasi::validate_wasi_component;
