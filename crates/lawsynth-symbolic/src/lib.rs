//! Bounded, deterministic scalar symbolic search.

mod config;
mod constants;
mod crossover;
mod error;
mod frontier;
mod grammar;
mod initialize;
mod mutate;
mod population;
mod search;
mod simplify;

pub use config::SymbolicConfig;
pub use constants::{CalibratedExpression, calibrate_affine};
pub use crossover::crossover_sum;
pub use error::SymbolicError;
pub use frontier::{ScoredExpression, pareto_by_loss_and_complexity};
pub use grammar::Grammar;
pub use initialize::initialize_population;
pub use mutate::replace_symbol;
pub use population::Population;
pub use search::enumerate;
pub use simplify::simplify_candidate;
