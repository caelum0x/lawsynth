//! Typed errors for state-estimator design and simulation.

use std::fmt;

use lawsynth_feedback::FeedbackError;
use lawsynth_koopman::KoopmanError;

/// Errors returned by observer design, Kalman-filter design, and simulation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EstimateError {
    /// A supplied matrix had zero rows or zero columns.
    EmptyMatrix,
    /// A matrix that must be square (the dynamics `A`) was rectangular.
    NonSquare,
    /// Two operands disagreed on shape (e.g. `C` columns ≠ `A` order, or an
    /// initial state / input of the wrong length).
    ShapeMismatch,
    /// A supplied value was not finite.
    NonFiniteValue,
    /// Ackermann observer placement was asked to design for more than one
    /// output. The output map `C` must be `1 × n` (the dual of single-input
    /// pole placement); a taller `C` needs dual robust placement.
    MultiOutput,
    /// The number of desired error poles did not equal the system order `n`.
    PoleCountMismatch,
    /// The desired error poles were not closed under complex conjugation, so the
    /// resulting observer gain would be complex rather than a real gain.
    NonRealDesignPoles,
    /// The pair `(A, C)` is not observable: the observability matrix
    /// `[C; CA; …; CAⁿ⁻¹]` is rank-deficient, so no gain can place the error
    /// poles. This is the dual of an uncontrollable feedback pair.
    Unobservable,
    /// The Kalman filter could not construct a stabilizing solution: `(A, C)` is
    /// not detectable (an unstable, unobservable mode). This is the dual of a
    /// non-stabilizable feedback pair.
    NotDetectable,
    /// A covariance matrix that must be symmetric was not.
    NotSymmetric,
    /// The measurement covariance `R` was not positive definite (hence not
    /// invertible).
    NotPositiveDefinite,
    /// The process covariance `Q` was not positive semidefinite.
    NotPositiveSemidefinite,
    /// The dual Riccati (Kleinman) iteration failed to converge in its budget.
    NoConvergence,
    /// A simulation was asked for a non-positive time step or zero steps.
    InvalidTimeStep,
    /// A feedback-design error with no distinct estimation counterpart.
    Feedback(FeedbackError),
    /// The shared deterministic eigensolver failed.
    Eigensolver(KoopmanError),
}

impl fmt::Display for EstimateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMatrix => write!(formatter, "matrix must have rows and columns"),
            Self::NonSquare => write!(formatter, "dynamics matrix A must be square"),
            Self::ShapeMismatch => write!(formatter, "operands have mismatched shapes"),
            Self::NonFiniteValue => write!(formatter, "matrix values must be finite"),
            Self::MultiOutput => {
                write!(formatter, "observer placement is single-output only (C must be 1×n)")
            }
            Self::PoleCountMismatch => {
                write!(formatter, "number of desired error poles must equal the system order")
            }
            Self::NonRealDesignPoles => {
                write!(formatter, "desired error poles must be closed under conjugation")
            }
            Self::Unobservable => {
                write!(formatter, "system is unobservable: observability matrix is rank-deficient")
            }
            Self::NotDetectable => {
                write!(formatter, "system is not detectable: an unstable mode is unobservable")
            }
            Self::NotSymmetric => write!(formatter, "covariance matrix must be symmetric"),
            Self::NotPositiveDefinite => {
                write!(formatter, "measurement covariance R must be symmetric positive definite")
            }
            Self::NotPositiveSemidefinite => {
                write!(formatter, "process covariance Q must be symmetric positive semidefinite")
            }
            Self::NoConvergence => write!(formatter, "dual Riccati iteration did not converge"),
            Self::InvalidTimeStep => {
                write!(formatter, "simulation requires dt > 0 and steps > 0")
            }
            Self::Feedback(inner) => write!(formatter, "feedback design failed: {inner}"),
            Self::Eigensolver(inner) => write!(formatter, "eigensolver failed: {inner}"),
        }
    }
}

impl std::error::Error for EstimateError {}

impl From<KoopmanError> for EstimateError {
    fn from(error: KoopmanError) -> Self {
        Self::Eigensolver(error)
    }
}

/// Maps a feedback-design error into its estimation dual.
///
/// Observer design is pole placement on `(Aᵀ, Cᵀ)`, and the Kalman filter is
/// LQR on `(Aᵀ, Cᵀ)`; the failure modes translate through that duality:
/// uncontrollable ↔ unobservable, non-stabilizable ↔ non-detectable, and a
/// multi-input request ↔ a multi-output map.
pub(crate) fn from_feedback(error: FeedbackError) -> EstimateError {
    match error {
        FeedbackError::EmptyMatrix => EstimateError::EmptyMatrix,
        FeedbackError::NonSquare => EstimateError::NonSquare,
        FeedbackError::ShapeMismatch => EstimateError::ShapeMismatch,
        FeedbackError::NonFiniteValue => EstimateError::NonFiniteValue,
        FeedbackError::MultiInput => EstimateError::MultiOutput,
        FeedbackError::PoleCountMismatch => EstimateError::PoleCountMismatch,
        FeedbackError::NonRealDesignPoles => EstimateError::NonRealDesignPoles,
        FeedbackError::Uncontrollable => EstimateError::Unobservable,
        FeedbackError::NotStabilizable => EstimateError::NotDetectable,
        FeedbackError::NotSymmetric => EstimateError::NotSymmetric,
        FeedbackError::NotPositiveDefinite => EstimateError::NotPositiveDefinite,
        FeedbackError::NotPositiveSemidefinite => EstimateError::NotPositiveSemidefinite,
        FeedbackError::NoConvergence => EstimateError::NoConvergence,
        FeedbackError::SingularSystem => EstimateError::Feedback(FeedbackError::SingularSystem),
        FeedbackError::Eigensolver(inner) => EstimateError::Eigensolver(inner),
    }
}
