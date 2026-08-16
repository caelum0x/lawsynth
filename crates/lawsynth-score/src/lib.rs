//! Deterministic scoring primitives for ranking discovery candidates.

mod complexity;
mod config;
mod dimensionality;
mod error;
mod fit;
mod metric;
mod pareto;
mod rank;
mod residual;
mod stability;

pub use complexity::expression_complexity;
pub use config::ScoringConfig;
pub use dimensionality::{InformationCriteria, information_criteria};
pub use error::ScoreError;
pub use fit::{FitStatistics, fit_statistics};
pub use metric::CandidateMetrics;
pub use pareto::pareto_front;
pub use rank::{rank_candidates, weighted_rank};
pub use residual::{ResidualSummary, residuals};
pub use stability::{SelectionStability, selection_stability};
