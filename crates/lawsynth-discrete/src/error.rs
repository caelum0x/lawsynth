//! Typed errors for discrete-time control and estimation.

use std::fmt;

use lawsynth_koopman::KoopmanError;

/// Errors returned by the discrete LQR, Kalman filter, and observer routines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiscreteError {
    /// A supplied matrix had zero rows or zero columns.
    EmptyMatrix,
    /// A matrix that must be square was rectangular.
    NonSquare,
    /// Two operands disagreed on shape (e.g. `B` rows ≠ `A` order).
    ShapeMismatch,
    /// A supplied value was not finite.
    NonFiniteValue,
    /// Observer pole placement was asked to design for more than one output.
    MultiOutput,
    /// The number of desired poles did not equal the system order `n`.
    PoleCountMismatch,
    /// The desired poles were not closed under complex conjugation, so the
    /// resulting gain would be complex rather than a real observer law.
    NonRealDesignPoles,
    /// The pair `(A, C)` is not observable, so no gain can place the error poles.
    Unobservable,
    /// A weight/covariance matrix that must be symmetric was not.
    NotSymmetric,
    /// The control/measurement weight `R` was not positive definite.
    NotPositiveDefinite,
    /// The state/process weight `Q` was not positive semidefinite.
    NotPositiveSemidefinite,
    /// A dense linear solve encountered a numerically singular system.
    SingularSystem,
    /// The DARE value iteration failed to converge within its budget — the pair
    /// is (numerically) not stabilizable/detectable, or the iterate diverged.
    NotConvergent,
    /// The shared deterministic eigensolver failed.
    Eigensolver(KoopmanError),
}

impl fmt::Display for DiscreteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMatrix => write!(formatter, "matrix must have rows and columns"),
            Self::NonSquare => write!(formatter, "matrix must be square"),
            Self::ShapeMismatch => write!(formatter, "operands have mismatched shapes"),
            Self::NonFiniteValue => write!(formatter, "matrix values must be finite"),
            Self::MultiOutput => {
                write!(formatter, "observer placement is single-output only (C must be 1×n)")
            }
            Self::PoleCountMismatch => {
                write!(formatter, "number of desired poles must equal the system order")
            }
            Self::NonRealDesignPoles => {
                write!(formatter, "desired poles must be closed under conjugation for a real gain")
            }
            Self::Unobservable => {
                write!(formatter, "system is unobservable: observability matrix is singular")
            }
            Self::NotSymmetric => write!(formatter, "weight/covariance matrix must be symmetric"),
            Self::NotPositiveDefinite => {
                write!(formatter, "weight R must be symmetric positive definite")
            }
            Self::NotPositiveSemidefinite => {
                write!(formatter, "weight Q must be symmetric positive semidefinite")
            }
            Self::SingularSystem => write!(formatter, "linear system is numerically singular"),
            Self::NotConvergent => {
                write!(formatter, "DARE iteration did not converge (not stabilizable/detectable)")
            }
            Self::Eigensolver(inner) => write!(formatter, "eigensolver failed: {inner}"),
        }
    }
}

impl std::error::Error for DiscreteError {}

impl From<KoopmanError> for DiscreteError {
    fn from(error: KoopmanError) -> Self {
        Self::Eigensolver(error)
    }
}
