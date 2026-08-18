//! Reproducible, dependency-free uncertainty quantification primitives.
//!
//! The crate operates on in-memory samples and deliberately exposes the
//! assumptions made by each calculation.  It does not manufacture probability
//! distributions or silently discard invalid values.

mod bootstrap;
mod coefficients;
mod config;
mod covariance;
mod error;
mod interval;
mod profile;
mod propagate;
mod samples;
mod source;
mod structural;

pub use bootstrap::{BootstrapResult, bootstrap};
pub use coefficients::{
    BootstrapCoefficientConfig, CoefficientEnsemble, ResampleMode, TermUncertainty,
    bootstrap_coefficients,
};
pub use config::{BootstrapConfig, IntervalConfig, PropagationConfig};
pub use covariance::CovarianceMatrix;
pub use error::UncertaintyError;
pub use interval::{confidence_interval, percentile};
pub use profile::{ProfilePoint, ProfileResult, profile_quadratic};
pub use propagate::{linear_propagate, monte_carlo_propagate};
pub use samples::Samples;
pub use source::{SourceKind, UncertaintySource};
pub use structural::{StructuralUncertainty, structural_score};
