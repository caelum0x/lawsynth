//! Typed errors for linear feedback design.

use std::fmt;

use lawsynth_koopman::KoopmanError;

/// Errors returned by the pole-placement and LQR design routines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedbackError {
    /// A supplied matrix had zero rows or zero columns.
    EmptyMatrix,
    /// A matrix that must be square was rectangular.
    NonSquare,
    /// Two operands disagreed on shape (e.g. `B` rows ≠ `A` order).
    ShapeMismatch,
    /// A supplied value was not finite.
    NonFiniteValue,
    /// Pole placement was asked to design for more than one input.
    MultiInput,
    /// The number of desired poles did not equal the system order `n`.
    PoleCountMismatch,
    /// The desired poles were not closed under complex conjugation, so the
    /// resulting gain would be complex rather than a real feedback law.
    NonRealDesignPoles,
    /// The pair `(A, b)` is not controllable, so no gain can place the poles.
    Uncontrollable,
    /// LQR could not construct an initial stabilizing gain: `(A, B)` is not
    /// stabilizable by the Bass bootstrap (effectively not controllable).
    NotStabilizable,
    /// A weight matrix that must be symmetric was not.
    NotSymmetric,
    /// The control weight `R` was not positive definite (hence not invertible).
    NotPositiveDefinite,
    /// The state weight `Q` was not positive semidefinite.
    NotPositiveSemidefinite,
    /// A dense linear solve encountered a numerically singular system.
    SingularSystem,
    /// The Kleinman/Riccati iteration failed to converge within its budget.
    NoConvergence,
    /// The shared deterministic eigensolver failed.
    Eigensolver(KoopmanError),
}

impl fmt::Display for FeedbackError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMatrix => write!(formatter, "matrix must have rows and columns"),
            Self::NonSquare => write!(formatter, "matrix must be square"),
            Self::ShapeMismatch => write!(formatter, "operands have mismatched shapes"),
            Self::NonFiniteValue => write!(formatter, "matrix values must be finite"),
            Self::MultiInput => {
                write!(formatter, "pole placement is single-input only (b must be n×1)")
            }
            Self::PoleCountMismatch => {
                write!(formatter, "number of desired poles must equal the system order")
            }
            Self::NonRealDesignPoles => {
                write!(formatter, "desired poles must be closed under conjugation for a real gain")
            }
            Self::Uncontrollable => {
                write!(formatter, "system is uncontrollable: controllability matrix is singular")
            }
            Self::NotStabilizable => {
                write!(formatter, "could not build an initial stabilizing gain (not stabilizable)")
            }
            Self::NotSymmetric => write!(formatter, "weight matrix must be symmetric"),
            Self::NotPositiveDefinite => {
                write!(formatter, "control weight R must be symmetric positive definite")
            }
            Self::NotPositiveSemidefinite => {
                write!(formatter, "state weight Q must be symmetric positive semidefinite")
            }
            Self::SingularSystem => write!(formatter, "linear system is numerically singular"),
            Self::NoConvergence => write!(formatter, "Riccati iteration did not converge"),
            Self::Eigensolver(inner) => write!(formatter, "eigensolver failed: {inner}"),
        }
    }
}

impl std::error::Error for FeedbackError {}

impl From<KoopmanError> for FeedbackError {
    fn from(error: KoopmanError) -> Self {
        Self::Eigensolver(error)
    }
}
