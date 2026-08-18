//! Errors produced while mapping basins of attraction.

use std::fmt;

use lawsynth_stability::StabilityError;

/// Every way basin mapping can fail, stated explicitly.
///
/// Basin mapping never fabricates an attractor and never forces a trajectory
/// into a basin it did not reach. Structural faults surface as typed errors;
/// honest per-trajectory outcomes (`Escaped`, `Undetermined`) are reported in the
/// [`crate::BasinReport`], not raised as errors.
#[derive(Clone, Debug, PartialEq)]
pub enum BasinError {
    /// The `states` slice was empty, so there is no vector field to integrate.
    EmptyStateSpace,
    /// The search box has a different dimension than `states`, so neither the
    /// initial-condition grid nor the flow can be formed.
    DimensionMismatch {
        /// The number of states supplied.
        states: usize,
        /// The number of intervals in the configured search box.
        search_box: usize,
    },
    /// A search interval was non-finite or inverted (`lower > upper`).
    InvalidSearchInterval {
        /// The offending axis index.
        index: usize,
        /// The interval's lower bound.
        lower: f64,
        /// The interval's upper bound.
        upper: f64,
    },
    /// A scalar configuration value was outside its valid range.
    InvalidConfig(&'static str),
    /// Locating the attractors via [`lawsynth_stability::analyze_stability`]
    /// failed. This carries through an unknown symbol (a non-autonomous field),
    /// a structural Jacobian fault, or an eigensolver failure.
    Stability(StabilityError),
}

impl fmt::Display for BasinError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyStateSpace => {
                write!(formatter, "the state space is empty; there is no vector field to integrate")
            }
            Self::DimensionMismatch { states, search_box } => write!(
                formatter,
                "search box has {search_box} intervals but there are {states} states"
            ),
            Self::InvalidSearchInterval { index, lower, upper } => write!(
                formatter,
                "search interval {index} is invalid: [{lower}, {upper}] must be finite with lower <= upper"
            ),
            Self::InvalidConfig(reason) => write!(formatter, "invalid basin config: {reason}"),
            Self::Stability(error) => write!(formatter, "attractor detection failed: {error}"),
        }
    }
}

impl std::error::Error for BasinError {}

impl From<StabilityError> for BasinError {
    fn from(error: StabilityError) -> Self {
        Self::Stability(error)
    }
}
