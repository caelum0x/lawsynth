use std::fmt;

use lawsynth_core::Identifier;
use lawsynth_expr::EvaluationError;
use lawsynth_jacobian::JacobianError;

/// Errors produced while assembling or integrating the forward-sensitivity
/// (variational) system.
///
/// Every failure mode is explicit. The integrator never fabricates a sensitivity
/// to paper over an ill-posed problem, and it never substitutes a default for a
/// symbol it was not given.
#[derive(Clone, Debug, PartialEq)]
pub enum SensitivityError {
    /// The `states` slice was empty, so there is no vector field to integrate.
    EmptyStateSpace,
    /// The initial-state vector has a different length than `states`.
    StateDimensionMismatch { states: usize, initial: usize },
    /// The parameter-value vector has a different length than `parameters`.
    ParameterDimensionMismatch { parameters: usize, values: usize },
    /// A non-finite value was supplied in the initial state or parameter values.
    NonFiniteInput { symbol: Identifier, value: f64 },
    /// The `parameters` slice listed the same identifier more than once, so the
    /// per-parameter sensitivity blocks would be ambiguous.
    DuplicateParameter(Identifier),
    /// An identifier appears in both `states` and `parameters`. A symbol cannot be
    /// simultaneously integrated as a state and held fixed as a parameter.
    ParameterIsState(Identifier),
    /// A field references a symbol that is neither a declared state nor a declared
    /// parameter. The variational system is defined only over `states ∪
    /// parameters`; a free symbol would have no value to bind, so it is rejected
    /// rather than guessed. This is the "unknown parameter symbol" case.
    UnknownSymbol(Identifier),
    /// A scalar configuration value (`t0`, `dt`, `steps`) was out of its valid
    /// range.
    InvalidConfig(&'static str),
    /// Assembling, differentiating, or evaluating the analytic Jacobian `J_x`
    /// failed (a duplicate or missing field, a duplicate state, an
    /// undifferentiable node, or a numeric evaluation error at a stage point).
    Jacobian(JacobianError),
    /// Numerically evaluating a field `f_i` or a parameter partial `∂f_i/∂θ_j`
    /// failed (an unknown symbol, a division by zero, a domain error, or a
    /// non-finite intermediate).
    Evaluation(EvaluationError),
}

impl fmt::Display for SensitivityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyStateSpace => {
                write!(formatter, "the state space is empty; there is no vector field to integrate")
            }
            Self::StateDimensionMismatch { states, initial } => write!(
                formatter,
                "initial state has {initial} components but there are {states} states"
            ),
            Self::ParameterDimensionMismatch { parameters, values } => write!(
                formatter,
                "parameter values has {values} entries but there are {parameters} parameters"
            ),
            Self::NonFiniteInput { symbol, value } => {
                write!(formatter, "value for '{symbol}' is not finite: {value}")
            }
            Self::DuplicateParameter(id) => {
                write!(formatter, "parameter '{id}' appears more than once")
            }
            Self::ParameterIsState(id) => {
                write!(formatter, "'{id}' is declared as both a state and a parameter")
            }
            Self::UnknownSymbol(symbol) => write!(
                formatter,
                "field references symbol '{symbol}', which is neither a state nor a parameter"
            ),
            Self::InvalidConfig(reason) => {
                write!(formatter, "invalid sensitivity config: {reason}")
            }
            Self::Jacobian(error) => write!(formatter, "jacobian error: {error}"),
            Self::Evaluation(error) => write!(formatter, "field evaluation failed: {error}"),
        }
    }
}

impl std::error::Error for SensitivityError {}

impl From<JacobianError> for SensitivityError {
    fn from(error: JacobianError) -> Self {
        Self::Jacobian(error)
    }
}

impl From<EvaluationError> for SensitivityError {
    fn from(error: EvaluationError) -> Self {
        Self::Evaluation(error)
    }
}
