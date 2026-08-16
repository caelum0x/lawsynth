//! The deterministic Phase 2 discovery pipeline.

mod assumptions;
mod branch;
mod cancellation;
mod candidate;
mod checkpoint;
mod config;
mod error;
mod execute;
mod graph;
mod plan;
mod stage;

pub use assumptions::{DependencyAssumptions, EdgeConstraint};
pub use branch::DiscoveryBranch;
pub use cancellation::CancellationToken;
pub use candidate::{DiscoveryCandidate, DiscoveryResult};
pub use checkpoint::DiscoveryCheckpoint;
pub use config::{DiscoveryConfig, SparseMethod};
pub use error::DiscoveryError;
pub use execute::{discover, discover_cancellable, discover_cancellable_with_checkpoint};
pub use graph::{DependencyEdge, DependencyGraph, infer_lagged_dependencies};
pub use plan::DiscoveryPlan;
pub use stage::DiscoveryStage;
