use std::fmt;

use lawsynth_core::Identifier;
use lawsynth_expr::EvaluationError;
use lawsynth_jacobian::JacobianError;

/// Errors produced while assembling or integrating the variational flow that
/// yields the Lyapunov spectrum.
///
/// Every failure mode is explicit. The estimator never fabricates an exponent to
/// paper over an ill-posed problem, never substitutes a default for a symbol it
/// was not given, and never returns a spectrum computed from a frame that has
/// numerically collapsed.
#[derive(Clone, Debug, PartialEq)]
pub enum LyapunovError {
    /// The `states` slice was empty, so there is no vector field to integrate.
    EmptyStateSpace,
    /// The initial-state vector has a different length than `states`.
    DimensionMismatch { states: usize, initial: usize },
    /// A non-finite value was supplied in the initial state.
    NonFiniteInput { symbol: Identifier, value: f64 },
    /// A field references a symbol that is not one of the declared states. The
    /// variational flow is defined only for an autonomous field over `states`; a
    /// free symbol would have no value to bind, so it is rejected rather than
    /// guessed.
    UnknownSymbol(Identifier),
    /// A scalar configuration value (`dt`, `steps`, the reorthonormalization
    /// interval, or the transient fraction) was out of its valid range.
    InvalidConfig(&'static str),
    /// Assembling, differentiating, or evaluating the analytic Jacobian `J(x)`
    /// failed (a duplicate or missing field, a duplicate state, an
    /// undifferentiable node, or a numeric evaluation error at a stage point).
    Jacobian(JacobianError),
    /// Numerically evaluating a field `f_i` failed (an unknown symbol, a division
    /// by zero, a domain error, or a non-finite intermediate).
    Evaluation(EvaluationError),
    /// The integrated state or perturbation frame became non-finite — the
    /// trajectory left every representable bound (a blow-up), so no meaningful
    /// exponent can be reported.
    NonFiniteState,
    /// A column of the perturbation frame collapsed to (numerically) zero length
    /// during reorthonormalization, so the Gram–Schmidt step cannot normalize it.
    /// This signals a degenerate or under-resolved variational flow.
    DegenerateFrame,
}

impl fmt::Display for LyapunovError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyStateSpace => {
                write!(formatter, "the state space is empty; there is no vector field to integrate")
            }
            Self::DimensionMismatch { states, initial } => write!(
                formatter,
                "initial state has {initial} components but there are {states} states"
            ),
            Self::NonFiniteInput { symbol, value } => {
                write!(formatter, "value for '{symbol}' is not finite: {value}")
            }
            Self::UnknownSymbol(symbol) => write!(
                formatter,
                "field references symbol '{symbol}', which is not one of the states; \
                 the field must be autonomous"
            ),
            Self::InvalidConfig(reason) => {
                write!(formatter, "invalid lyapunov config: {reason}")
            }
            Self::Jacobian(error) => write!(formatter, "jacobian error: {error}"),
            Self::Evaluation(error) => write!(formatter, "field evaluation failed: {error}"),
            Self::NonFiniteState => {
                write!(formatter, "integration produced a non-finite state or frame (blow-up)")
            }
            Self::DegenerateFrame => write!(
                formatter,
                "a perturbation-frame column collapsed to zero length during reorthonormalization"
            ),
        }
    }
}

impl std::error::Error for LyapunovError {}

impl From<JacobianError> for LyapunovError {
    fn from(error: JacobianError) -> Self {
        Self::Jacobian(error)
    }
}

impl From<EvaluationError> for LyapunovError {
    fn from(error: EvaluationError) -> Self {
        Self::Evaluation(error)
    }
}
