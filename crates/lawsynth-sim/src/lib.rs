//! Deterministic continuous-time simulation for World IR.

mod compile;
mod config;
mod context;
mod discrete;
mod error;
mod hybrid;
mod interpreter;
mod ode;
mod request;
mod sde;
mod state;
mod trajectory;

pub use compile::{CompiledContinuousWorld, CompiledDiscreteWorld};
pub use config::SimulationLimits;
pub use context::SimulationContext;
pub use discrete::simulate_discrete;
pub use error::SimulationError;
pub use hybrid::split_at_events;
pub use interpreter::{evaluate_continuous, evaluate_discrete};
pub use ode::simulate;
pub use request::{DiscreteSimulationConfig, ScheduledValue, SimulationConfig, SimulationRequest};
pub use sde::{SdeConfig, SdeTrajectory, euler_maruyama};
pub use trajectory::Trajectory;
