//! Typed errors for parameter continuation and bifurcation detection.

use std::fmt;

use lawsynth_core::Identifier;
use lawsynth_stability::StabilityError;

/// Errors produced while sweeping a parameter and tracking bifurcations.
///
/// Every failure mode is explicit. The continuation never fabricates a branch or
/// a bifurcation to paper over an ill-posed sweep; a per-step stability failure
/// surfaces as [`BifurcationError::Stability`] with the offending parameter
/// value, rather than being silently dropped.
#[derive(Clone, Debug, PartialEq)]
pub enum BifurcationError {
    /// The `states` slice was empty, so there is no vector field to continue.
    EmptyStateSpace,
    /// The continuation parameter is also listed as a state. A symbol cannot be
    /// both an evolving coordinate and the swept parameter.
    ParameterIsState(Identifier),
    /// A `Sweep` field was out of its valid range.
    InvalidSweep(&'static str),
    /// Locating or classifying fixed points at a particular parameter value
    /// failed. The wrapped error carries the underlying stability fault.
    Stability { parameter_value: f64, source: StabilityError },
}

impl fmt::Display for BifurcationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyStateSpace => {
                write!(formatter, "the state space is empty; there is no vector field to continue")
            }
            Self::ParameterIsState(parameter) => write!(
                formatter,
                "parameter '{parameter}' is also a state; it cannot be both swept and evolved"
            ),
            Self::InvalidSweep(reason) => write!(formatter, "invalid sweep: {reason}"),
            Self::Stability { parameter_value, source } => {
                write!(
                    formatter,
                    "stability analysis failed at parameter {parameter_value}: {source}"
                )
            }
        }
    }
}

impl std::error::Error for BifurcationError {}
