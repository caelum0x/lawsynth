//! The deterministic Phase 2 discovery pipeline.

mod assumptions;
mod branch;
mod cancellation;
mod candidate;
mod causal;
mod checkpoint;
mod config;
mod error;
mod execute;
mod graph;
mod pareto;
mod plan;
mod refine;
mod stage;

pub use assumptions::{DependencyAssumptions, EdgeConstraint};
pub use branch::DiscoveryBranch;
pub use cancellation::CancellationToken;
pub use candidate::{DiscoveryCandidate, DiscoveryResult, ParameterRefinement};
pub use checkpoint::DiscoveryCheckpoint;
pub use config::{CausalHypothesisConfig, DiscoveryConfig, RefinementConfig, SparseMethod};
pub use error::DiscoveryError;
pub use execute::{discover, discover_cancellable, discover_cancellable_with_checkpoint};
pub use graph::{DependencyEdge, DependencyGraph, infer_lagged_dependencies};
pub use pareto::{CandidateScore, frontier_of, pareto_frontier};
pub use plan::DiscoveryPlan;
pub use stage::DiscoveryStage;

// Re-exported from `lawsynth-causal` so callers can inspect the §8.6 dependency
// hypothesis (and the assumptions it rests on) without a direct dependency.
pub use lawsynth_causal::{AssumptionSet, CausalAssumption, CausalGraph, Edge as CausalEdge};
