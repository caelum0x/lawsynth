use std::fmt;

use lawsynth_sparse::SparseError;

/// Everything that can go wrong while discovering an evolution PDE from a field.
#[derive(Clone, Debug, PartialEq)]
pub enum PdeError {
    /// The configuration itself is malformed (bad orders, empty library, etc.).
    InvalidConfig(String),
    /// A spatial or temporal step is not a finite, strictly positive number.
    InvalidStep(String),
    /// The field has no rows (or its first row has no columns).
    EmptyField,
    /// The field is not rectangular: `row` has `found` columns, not `expected`.
    NonRectangularField { row: usize, expected: usize, found: usize },
    /// A field sample is not finite (`NaN`/`±inf`).
    NonFiniteValue { row: usize, col: usize },
    /// The grid is too small along `axis` for the required central stencil: the
    /// interior would be empty. Central time differencing needs at least three
    /// snapshots; a spatial stencil of half-width `h` needs at least `2h + 1`
    /// columns.
    TooFewPoints { axis: &'static str, have: usize, need: usize },
    /// The field does not evolve in time (the time derivative is ~0 everywhere),
    /// so there is no dynamics to regress against.
    DegenerateField,
    /// The sparse regression over the flattened interior failed.
    Sparse(SparseError),
    /// An internal invariant was violated while assembling the problem.
    Internal(String),
}

impl fmt::Display for PdeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(reason) => {
                write!(formatter, "invalid PDE discovery configuration: {reason}")
            }
            Self::InvalidStep(reason) => write!(formatter, "invalid grid step: {reason}"),
            Self::EmptyField => {
                write!(formatter, "field must have at least one row and one column")
            }
            Self::NonRectangularField { row, expected, found } => write!(
                formatter,
                "field is not rectangular: row {row} has {found} columns, expected {expected}"
            ),
            Self::NonFiniteValue { row, col } => {
                write!(formatter, "field value at (row {row}, col {col}) is not finite")
            }
            Self::TooFewPoints { axis, have, need } => write!(
                formatter,
                "too few {axis} points for the central stencil: have {have}, need at least {need}"
            ),
            Self::DegenerateField => {
                write!(formatter, "field does not evolve in time; there is no dynamics to discover")
            }
            Self::Sparse(error) => write!(formatter, "sparse regression error: {error}"),
            Self::Internal(reason) => write!(formatter, "internal PDE discovery error: {reason}"),
        }
    }
}

impl std::error::Error for PdeError {}

impl From<SparseError> for PdeError {
    fn from(error: SparseError) -> Self {
        Self::Sparse(error)
    }
}
