use std::fmt;

/// Errors returned by the Koopman/DMD operator-discovery boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KoopmanError {
    /// A snapshot matrix had zero rows or zero columns.
    EmptyMatrix,
    /// `X` and `X'` (or a control matrix) disagreed on shape.
    ShapeMismatch,
    /// A supplied value was not finite.
    NonFiniteValue,
    /// The requested truncation rank was zero or exceeded `min(rows, cols)`.
    InvalidRank,
    /// The dataset did not contain enough snapshots to form a pair.
    InsufficientSnapshots,
    /// A dictionary produced no lifted features or an inconsistent width.
    InvalidDictionary,
    /// The iterative linear algebra failed to converge within its budget.
    NoConvergence,
    /// A linear system encountered during the fit was numerically singular.
    SingularSystem,
}

impl fmt::Display for KoopmanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMatrix => write!(formatter, "snapshot matrix must have rows and columns"),
            Self::ShapeMismatch => write!(formatter, "snapshot matrices have mismatched shapes"),
            Self::NonFiniteValue => write!(formatter, "snapshot values must be finite"),
            Self::InvalidRank => {
                write!(formatter, "truncation rank must be in 1..=min(rows, cols)")
            }
            Self::InsufficientSnapshots => {
                write!(formatter, "at least two aligned snapshots are required")
            }
            Self::InvalidDictionary => {
                write!(formatter, "feature dictionary produced no usable columns")
            }
            Self::NoConvergence => {
                write!(formatter, "iterative decomposition did not converge")
            }
            Self::SingularSystem => write!(formatter, "linear system is numerically singular"),
        }
    }
}

impl std::error::Error for KoopmanError {}
