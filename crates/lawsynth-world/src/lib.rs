//! Typed World IR for executable continuous-time models.

mod config;
mod error;
mod event;
mod graph;
mod intervention;
mod law;
mod parameter;
mod regime;
mod variable;
mod world;

pub use config::WorldConfig;
pub use error::WorldError;
pub use event::{Event, EventDirection, crosses_zero};
pub use graph::expression_symbols;
pub use intervention::{Intervention, InterventionTarget};
pub use law::{ContinuousLaw, DiscreteLaw};
pub use lawsynth_units::Unit;
pub use parameter::Parameter;
pub use regime::{RegimeError, RegimeInterval, RegimeSchedule};
pub use variable::{Variable, VariableRole};
pub use world::{DiscreteWorld, World};
