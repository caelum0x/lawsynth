use std::fmt;

use lawsynth_core::Identifier;
use lawsynth_jacobian::JacobianError;
use lawsynth_koopman::KoopmanError;

/// Errors produced while locating and classifying fixed points.
///
/// Every failure mode is explicit. The analysis never fabricates a root to
/// paper over an ill-posed problem, and it never substitutes a default for a
/// symbol it was not given.
#[derive(Clone, Debug, PartialEq)]
pub enum StabilityError {
    /// The `states` slice was empty, so there is no vector field to analyse.
    EmptyStateSpace,
    /// The configured search box has a different dimension than `states`, so the
    /// per-axis seed lattice cannot be formed.
    DimensionMismatch { states: usize, search_box: usize },
    /// A search interval was non-finite or inverted (`lower > upper`).
    InvalidSearchInterval { index: usize, lower: f64, upper: f64 },
    /// A scalar configuration value was out of its valid range.
    InvalidConfig(&'static str),
    /// Assembling or evaluating the analytic Jacobian failed (a duplicate or
    /// missing field, a duplicate state, an undifferentiable node, or a numeric
    /// evaluation error at a candidate point).
    Jacobian(JacobianError),
    /// A field references a symbol that is not one of the states. The analysis
    /// is defined only for autonomous fields `ẋ = f(x)`; a free parameter would
    /// have no value to evaluate at, so this is rejected rather than guessed.
    UnknownSymbol(Identifier),
    /// The deterministic eigensolver could not decompose the Jacobian at a
    /// located fixed point (non-finite entry or non-convergence).
    Eigen(KoopmanError),
}

impl fmt::Display for StabilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyStateSpace => {
                write!(formatter, "the state space is empty; there is no vector field to analyse")
            }
            Self::DimensionMismatch { states, search_box } => write!(
                formatter,
                "search box has {search_box} intervals but there are {states} states"
            ),
            Self::InvalidSearchInterval { index, lower, upper } => write!(
                formatter,
                "search interval {index} is invalid: [{lower}, {upper}] must be finite with lower <= upper"
            ),
            Self::InvalidConfig(reason) => write!(formatter, "invalid stability config: {reason}"),
            Self::Jacobian(error) => write!(formatter, "jacobian error: {error}"),
            Self::UnknownSymbol(symbol) => write!(
                formatter,
                "field references symbol '{symbol}', which is not one of the states"
            ),
            Self::Eigen(error) => write!(formatter, "eigendecomposition failed: {error}"),
        }
    }
}

impl std::error::Error for StabilityError {}

impl From<JacobianError> for StabilityError {
    fn from(error: JacobianError) -> Self {
        Self::Jacobian(error)
    }
}
