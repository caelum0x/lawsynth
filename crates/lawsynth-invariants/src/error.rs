use std::fmt;

use lawsynth_core::Identifier;
use lawsynth_expr::EvaluationError;
use lawsynth_jacobian::JacobianError;
use lawsynth_koopman::KoopmanError;

/// Errors returned by the conserved-quantity detection boundary.
#[derive(Clone, Debug, PartialEq)]
pub enum InvariantError {
    /// The caller supplied no state variables to search over.
    NoStates,
    /// The caller supplied no vector-field expressions.
    EmptyFields,
    /// A state identifier appeared more than once in `states`.
    DuplicateState(Identifier),
    /// A state has no matching right-hand-side field expression.
    MissingField(Identifier),
    /// A field references a symbol that is not one of the declared states.
    UnknownSymbol(Identifier),
    /// The requested monomial degree was zero (only the excluded constant).
    InvalidDegree,
    /// The sample-grid resolution was fewer than two points per axis.
    InvalidResolution,
    /// The sample box was degenerate (`lo >= hi`) or non-finite.
    InvalidBox,
    /// The singular-value tolerance was negative or non-finite.
    InvalidTolerance,
    /// The library after excluding the constant contained no basis functions.
    EmptyLibrary,
    /// The deterministic sample grid held fewer points than basis functions, so
    /// the numerical nullspace would be underdetermined.
    InsufficientSamples { samples: usize, basis: usize },
    /// Symbolically differentiating a basis function failed.
    Differentiation(JacobianError),
    /// Numerically evaluating a Lie derivative at a grid point failed.
    Evaluation(EvaluationError),
    /// The dense linear-algebra layer (matrix build or SVD) failed.
    LinearAlgebra(KoopmanError),
}

impl fmt::Display for InvariantError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoStates => write!(formatter, "at least one state variable is required"),
            Self::EmptyFields => write!(formatter, "at least one field expression is required"),
            Self::DuplicateState(id) => {
                write!(formatter, "state '{id}' appears more than once")
            }
            Self::MissingField(id) => {
                write!(formatter, "no field expression was supplied for state '{id}'")
            }
            Self::UnknownSymbol(id) => {
                write!(formatter, "a field references symbol '{id}', which is not a declared state")
            }
            Self::InvalidDegree => write!(formatter, "monomial degree must be at least 1"),
            Self::InvalidResolution => {
                write!(formatter, "sample resolution must be at least 2 points per axis")
            }
            Self::InvalidBox => {
                write!(formatter, "sample box must be finite with lower bound below upper bound")
            }
            Self::InvalidTolerance => {
                write!(formatter, "singular-value tolerance must be finite and non-negative")
            }
            Self::EmptyLibrary => {
                write!(formatter, "the candidate library is empty after excluding the constant")
            }
            Self::InsufficientSamples { samples, basis } => write!(
                formatter,
                "sample grid has {samples} points but the library has {basis} functions; \
                 the nullspace would be underdetermined"
            ),
            Self::Differentiation(error) => {
                write!(formatter, "failed to differentiate a basis function: {error}")
            }
            Self::Evaluation(error) => {
                write!(formatter, "failed to evaluate a Lie derivative: {error}")
            }
            Self::LinearAlgebra(error) => write!(formatter, "linear algebra failed: {error}"),
        }
    }
}

impl std::error::Error for InvariantError {}

impl From<JacobianError> for InvariantError {
    fn from(error: JacobianError) -> Self {
        Self::Differentiation(error)
    }
}

impl From<EvaluationError> for InvariantError {
    fn from(error: EvaluationError) -> Self {
        Self::Evaluation(error)
    }
}

impl From<KoopmanError> for InvariantError {
    fn from(error: KoopmanError) -> Self {
        Self::LinearAlgebra(error)
    }
}
