//! Causal structure utilities with deterministic, dependency-free estimators.
//!
//! The crate deliberately exposes diagnostics rather than causal claims: a
//! Granger score is predictive evidence, and graph validity encodes declared
//! assumptions rather than discovering them from observational data.

pub mod assumptions;
pub mod config;
pub mod equivalence;
pub mod error;
pub mod granger;
pub mod graph;
pub mod independence;
pub mod lagged;
pub mod sensitivity;
pub mod time_order;

pub use assumptions::{AssumptionSet, CausalAssumption};
pub use config::CausalConfig;
pub use equivalence::{MarkovEquivalence, equivalence_class};
pub use error::{CausalError, Result};
pub use granger::{GrangerResult, granger_test};
pub use graph::{CausalGraph, Edge};
pub use independence::{IndependenceResult, pearson_independence};
pub use lagged::{LaggedObservation, lagged_observations};
pub use sensitivity::{ConfoundingBound, e_value};
pub use time_order::{TimeOrder, validate_time_order};
