//! The deterministic Phase 2 discovery pipeline.

mod assumptions;
mod branch;
mod cancellation;
mod candidate;
mod causal;
mod checkpoint;
mod coefficients;
mod config;
mod distributed;
mod error;
mod execute;
mod graph;
mod pareto;
mod plan;
mod refine;
mod stage;
mod template;

pub use assumptions::{DependencyAssumptions, EdgeConstraint};
pub use branch::DiscoveryBranch;
pub use cancellation::CancellationToken;
pub use candidate::{
    DimensionalPruningReport, DiscoveryCandidate, DiscoveryResult, ParameterRefinement,
};
pub use checkpoint::DiscoveryCheckpoint;
pub use coefficients::StateCoefficientEnsemble;
pub use config::{
    CausalHypothesisConfig, DimensionalUnits, DiscoveryConfig, RefinementConfig, SparseMethod,
};
pub use distributed::{discover_partitioned, evaluate_library_partitioned};
pub use error::DiscoveryError;
pub use execute::{discover, discover_cancellable, discover_cancellable_with_checkpoint};
pub use graph::{DependencyEdge, DependencyGraph, infer_lagged_dependencies};
pub use pareto::{CandidateScore, frontier_of, pareto_frontier};
pub use plan::DiscoveryPlan;
pub use stage::DiscoveryStage;
pub use template::{
    DropReason, DroppedTerm, TemplateError, TemplateFilterReport, TemplatePrior, TemplateSelection,
    TermKind,
};

// Re-exported from `lawsynth-causal` so callers can inspect the §8.6 dependency
// hypothesis (and the assumptions it rests on) without a direct dependency.
pub use lawsynth_causal::{AssumptionSet, CausalAssumption, CausalGraph, Edge as CausalEdge};
