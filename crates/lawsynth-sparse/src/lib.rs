//! Deterministic sparse regression algorithms for candidate equation fitting.

mod config;
mod constrained;
mod error;
mod frols;
mod group;
mod lasso;
mod problem;
mod sr3;
mod ssr;
mod stability;
mod standardize;
mod stlsq;
mod trapping;

pub use config::SparseConfig;
pub use constrained::{NonnegativeConfig, nonnegative_least_squares};
pub use error::SparseError;
pub use frols::{FrolsStep, frols};
pub use group::{GroupConfig, group_stlsq};
pub use lasso::{LassoConfig, lasso};
pub use problem::RegressionProblem;
pub use sr3::sr3;
pub use ssr::{SsrStep, ssr};
pub use stability::{StabilityConfig, StabilitySelection, stability_selection};
use standardize::restore_solution;
pub use standardize::{FeatureScaling, standardize_problem};
pub use stlsq::{SparseSolution, stlsq};
pub use trapping::{TrappingConfig, trapping};

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

/// Fits FROLS after deterministic root-mean-square feature scaling.
pub fn frols_standardized(
    problem: &RegressionProblem,
    config: &SparseConfig,
) -> Result<SparseSolution, SparseError> {
    let (scaled, scaling) = standardize_problem(problem)?;
    let solution = frols(&scaled, config)?;
    restore_solution(problem, solution, &scaling)
}

/// Fits SSR after deterministic root-mean-square feature scaling.
pub fn ssr_standardized(
    problem: &RegressionProblem,
    config: &SparseConfig,
) -> Result<SparseSolution, SparseError> {
    let (scaled, scaling) = standardize_problem(problem)?;
    let solution = ssr(&scaled, config)?;
    restore_solution(problem, solution, &scaling)
}

/// Fits the stability-biased trapping variant after deterministic root-mean-square
/// feature scaling. RMS scaling uses strictly positive scales, so the sign of the
/// diagonal self-feedback term — and therefore the stability bias — is preserved.
pub fn trapping_standardized(
    problem: &RegressionProblem,
    config: &TrappingConfig,
) -> Result<SparseSolution, SparseError> {
    let (scaled, scaling) = standardize_problem(problem)?;
    let solution = trapping(&scaled, config)?;
    restore_solution(problem, solution, &scaling)
}
