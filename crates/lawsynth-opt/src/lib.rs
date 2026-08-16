//! Small deterministic optimizers used to calibrate symbolic candidates.

mod bounds;
mod config;
mod coordinate;
mod error;
mod lbfgs;
mod least_squares;
mod mixed;
mod nelder_mead;
mod objective;
mod termination;

pub use bounds::ParameterBounds;
pub use config::CoordinateConfig;
pub use coordinate::{CoordinateResult, coordinate_minimize};
pub use error::OptimizationError;
pub use lbfgs::{LbfgsConfig, lbfgs_minimize};
pub use least_squares::{AffineFit, fit_affine};
pub use mixed::{MixedResult, mixed_minimize};
pub use nelder_mead::{NelderMeadConfig, nelder_mead_minimize};
pub use objective::{mean_squared_error, residual_sum_squares};
pub use termination::TerminationReason;
