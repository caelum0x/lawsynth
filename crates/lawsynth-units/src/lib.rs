//! Small, deterministic SI dimensional-analysis primitives.

mod buckingham;
mod builtins;
mod check;
mod config;
mod convert;
mod dimension;
mod error;
mod infer;
mod parse;
mod registry;
mod search;
mod unit;

pub use buckingham::dimensionless_groups;
pub use builtins::builtin_registry;
pub use check::require_compatible;
pub use config::UnitConfig;
pub use convert::convert;
pub use dimension::Dimension;
pub use error::UnitError;
pub use infer::infer_expression_dimension;
pub use parse::parse_unit;
pub use registry::UnitRegistry;
pub use search::{DimensionTerm, admits_dimension, admits_scaled_dimension, infer_term_dimension};
pub use unit::Unit;
