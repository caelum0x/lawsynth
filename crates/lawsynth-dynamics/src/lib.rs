//! Dataset-backed continuous and discrete system-identification problems.

mod config;
mod continuous;
mod control;
mod delay;
mod discrete;
mod error;
mod implicit;
mod problem;
mod refine;
mod result;

pub use config::DynamicsConfig;
pub use continuous::ContinuousProblem;
pub use control::ControlledProblem;
pub use delay::{DelaySamples, DelayedProblem};
pub use discrete::DiscreteProblem;
pub use error::DynamicsError;
pub use implicit::ImplicitProblem;
pub use problem::IdentificationProblem;
pub use refine::central_derivative;
pub use result::{DiscreteTransitions, discrete_transitions};
