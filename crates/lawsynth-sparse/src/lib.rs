//! Deterministic sparse regression algorithms for candidate equation fitting.

mod config;
mod constrained;
mod error;
mod group;
mod lasso;
mod problem;
mod sr3;
mod stability;
mod standardize;
mod stlsq;

pub use config::SparseConfig;
pub use constrained::{NonnegativeConfig, nonnegative_least_squares};
pub use error::SparseError;
pub use group::{GroupConfig, group_stlsq};
pub use lasso::{LassoConfig, lasso};
pub use problem::RegressionProblem;
pub use sr3::sr3;
pub use stability::{StabilityConfig, StabilitySelection, stability_selection};
use standardize::restore_solution;
pub use standardize::{FeatureScaling, standardize_problem};
pub use stlsq::{SparseSolution, stlsq};

/// Fits STLSQ after deterministic root-mean-square feature scaling.
///
/// The returned coefficients always correspond to the caller's original
/// feature matrix, so downstream expression construction needs no special
/// handling.
pub fn stlsq_standardized(
    problem: &RegressionProblem,
    config: &SparseConfig,
) -> Result<SparseSolution, SparseError> {
    let (scaled, scaling) = standardize_problem(problem)?;
    let solution = stlsq(&scaled, config)?;
    restore_solution(problem, solution, &scaling)
}

/// Fits SR3 after deterministic root-mean-square feature scaling.
pub fn sr3_standardized(
    problem: &RegressionProblem,
    config: &SparseConfig,
) -> Result<SparseSolution, SparseError> {
    let (scaled, scaling) = standardize_problem(problem)?;
    let solution = sr3(&scaled, config)?;
    restore_solution(problem, solution, &scaling)
}
