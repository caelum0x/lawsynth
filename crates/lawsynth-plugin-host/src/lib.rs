//! Isolated plugin host controls.  This crate never grants filesystem or
//! network access implicitly and uses process boundaries for executable plugins.

mod config;
mod discover;
mod error;
mod lifecycle;
mod permissions;
mod process;
mod registry;
mod resources;
mod rpc;
mod wasi;

pub use config::HostConfig;
pub use discover::discover_manifests;
pub use error::HostError;
pub use lifecycle::HostedPlugin;
pub use permissions::{PermissionPolicy, PermissionSet};
pub use process::{ProcessHandle, ProcessSpec};
pub use registry::PluginRegistry;
pub use resources::ResourceMeter;
pub use rpc::{RpcChannel, read_frame, write_frame};
pub use wasi::validate_wasi_component;
