//! Stable, dependency-free extension protocol primitives.
//!
//! This crate intentionally exposes data and protocol contracts rather than a
//! Rust ABI.  Plugins may run as WASI components or isolated processes; every
//! input is validated before the host allocates or executes work.

mod algorithm;
mod capability;
mod config;
mod data_adapter;
mod error;
mod lifecycle;
mod limits;
mod manifest;
mod protocol;
mod simulator;

pub use algorithm::{AlgorithmPlugin, AlgorithmRequest, AlgorithmResponse};
pub use capability::{Capability, CapabilitySet};
pub use config::ProtocolConfig;
pub use data_adapter::{
    Column, DataAdapter, DataBatch, DataSchema, ScalarType, validate_row_group,
};
pub use error::PluginError;
pub use lifecycle::{LifecycleEvent, PluginState};
pub use limits::ResourceLimits;
pub use manifest::{PluginKind, PluginManifest};
pub use protocol::{Frame, FrameKind, MAX_FRAME_BYTES, PROTOCOL_VERSION};
pub use simulator::{SimulationPlugin, SimulationRequest, SimulationResponse};
