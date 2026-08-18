use std::fmt;

use lawsynth_core::Identifier;
use lawsynth_expr::EvaluationError;

/// Errors produced while building or evaluating an analytic Jacobian.
///
/// Every failure mode is explicit: symbolic differentiation never silently
/// returns a zero for a node it cannot handle, and assembly never guesses at a
/// missing or ambiguous field.
#[derive(Clone, Debug, PartialEq)]
pub enum JacobianError {
    /// The `states` slice listed the same identifier more than once, so the row
    /// and column ordering would be ambiguous.
    DuplicateState(Identifier),
    /// Two field entries share the same derivative-target identifier, so the row
    /// expression for that state would be ambiguous.
    DuplicateField(Identifier),
    /// A state in `states` has no corresponding field `dx_i/dt = f_i`, so its
    /// Jacobian row cannot be formed.
    MissingField(Identifier),
    /// A node could not be differentiated in closed real form. Currently this is
    /// only reachable for a power `b^g` whose base `b` is a non-positive constant
    /// and whose exponent `g` depends on the differentiation variable — the
    /// generalized power rule would require `log(b)` of a non-positive base. The
    /// variant is also a forward-compatible guard against future IR node kinds.
    UnsupportedDerivative { node: String, reason: &'static str },
    /// Numeric evaluation of a Jacobian entry failed (e.g. an unknown symbol was
    /// not supplied, a division by zero, or a domain error).
    Evaluation(EvaluationError),
}

impl fmt::Display for JacobianError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateState(id) => {
                write!(formatter, "state '{id}' appears more than once in the ordering")
            }
            Self::DuplicateField(id) => {
                write!(formatter, "field target '{id}' is defined more than once")
            }
            Self::MissingField(id) => {
                write!(formatter, "no field 'd{id}/dt = f' was supplied for state '{id}'")
            }
            Self::UnsupportedDerivative { node, reason } => {
                write!(formatter, "cannot differentiate node '{node}': {reason}")
            }
            Self::Evaluation(error) => write!(formatter, "jacobian evaluation failed: {error}"),
        }
    }
}

impl std::error::Error for JacobianError {}

impl From<EvaluationError> for JacobianError {
    fn from(error: EvaluationError) -> Self {
        Self::Evaluation(error)
    }
}
