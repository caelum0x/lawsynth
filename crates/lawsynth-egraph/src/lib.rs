//! A bounded equivalence graph for canonical scalar LawSynth expressions.

mod analysis;
mod config;
mod cost;
mod error;
mod extract;
mod graph;
mod language;
mod limits;
mod proof;
mod rules;
mod schedule;

pub use analysis::ExpressionAnalysis;
pub use config::RewriteConfig;
pub use cost::expression_cost;
pub use error::RewriteError;
pub use extract::extract_lowest_cost;
pub use graph::{EquivalenceClass, EquivalenceGraph};
pub use language::ExpressionLanguage;
pub use limits::RewriteLimits;
pub use proof::RewriteProof;
pub use rules::{RewriteRule, normalize};
pub use schedule::RewriteSchedule;
