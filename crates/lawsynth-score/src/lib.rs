//! Deterministic scoring primitives for ranking discovery candidates.

mod complexity;
mod config;
mod dimensionality;
mod error;
mod fit;
mod mdl;
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
pub use mdl::{
    DescriptionLength, ModelDescription, OPERATOR_VOCABULARY, description_length, most_parsimonious,
};
pub use metric::CandidateMetrics;
pub use pareto::{pareto_front, pareto_front_2d};
pub use rank::{rank_candidates, weighted_rank};
pub use residual::{ResidualSummary, residuals};
pub use stability::{SelectionStability, selection_stability};
