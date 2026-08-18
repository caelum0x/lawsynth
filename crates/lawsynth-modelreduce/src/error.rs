//! Typed errors for the model-reduction boundary.

use std::fmt;

/// Errors returned by the balanced-truncation model-reduction boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelReduceError {
    /// A supplied matrix had zero rows or zero columns.
    EmptyMatrix,
    /// The state matrix `A` was not square.
    NonSquareState,
    /// The input matrix `B` did not have `n` rows (one per state).
    InputDimensionMismatch,
    /// The output matrix `C` did not have `n` columns (one per state).
    OutputDimensionMismatch,
    /// `A` is not Hurwitz (some eigenvalue has non-negative real part), so the
    /// controllability/observability gramians do not exist.
    NotStable,
    /// The requested truncation order was zero or exceeded the state dimension.
    InvalidOrder,
    /// The energy tolerance was not a finite value in `[0, 1)`.
    InvalidTolerance,
    /// A gramian was numerically singular — the realization is (near) minimal in
    /// a way that leaves the balancing transform undefined (a zero Hankel
    /// singular value: a mode that is both uncontrollable and unobservable).
    SingularSystem,
    /// A deterministic iterative decomposition did not converge in its budget.
    NoConvergence,
    /// An internal matrix operation received mismatched shapes (a bug guard).
    ShapeMismatch,
}

impl fmt::Display for ModelReduceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMatrix => write!(formatter, "a supplied matrix has zero rows or columns"),
            Self::NonSquareState => write!(formatter, "the state matrix A must be square"),
            Self::InputDimensionMismatch => {
                write!(formatter, "the input matrix B must have one row per state")
            }
            Self::OutputDimensionMismatch => {
                write!(formatter, "the output matrix C must have one column per state")
            }
            Self::NotStable => {
                write!(formatter, "A must be Hurwitz (all eigenvalues with Re < 0) to reduce")
            }
            Self::InvalidOrder => {
                write!(formatter, "the reduced order must be in 1..=n")
            }
            Self::InvalidTolerance => {
                write!(formatter, "the energy tolerance must be finite and in [0, 1)")
            }
            Self::SingularSystem => {
                write!(formatter, "a gramian is singular; the balancing transform is undefined")
            }
            Self::NoConvergence => {
                write!(formatter, "a deterministic decomposition did not converge")
            }
            Self::ShapeMismatch => write!(formatter, "internal matrix shapes disagree"),
        }
    }
}

impl std::error::Error for ModelReduceError {}
