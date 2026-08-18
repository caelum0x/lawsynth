use std::fmt;

use lawsynth_core::Identifier;
use lawsynth_sparse::SparseError;

/// Everything that can go wrong while discovering an SDE from sample paths.
#[derive(Clone, Debug, PartialEq)]
pub enum SdeError {
    /// The configuration itself is malformed (non-positive bins, degree, etc.).
    InvalidConfig(String),
    /// A requested state column is not present in the dataset.
    UnknownStateColumn(Identifier),
    /// Fewer than two rows: no increment `ΔX` can be formed.
    TooFewSamples { rows: usize },
    /// `require_regular_time` is set but the timestamps are not evenly spaced.
    IrregularTimeAxis,
    /// Every observed value of a state is (numerically) identical, so the state
    /// space cannot be partitioned into bins.
    DegenerateState { state: Identifier },
    /// After binning, too few bins met `min_bin_count` to fit the library — the
    /// sparse regression would be under-determined.
    TooFewPopulatedBins { state: Identifier, populated: usize, required: usize },
    /// Building or evaluating the candidate feature library failed.
    Feature(String),
    /// The sparse regression over the binned estimates failed.
    Sparse(SparseError),
    /// An internal invariant on the reconstructed data was violated.
    Internal(String),
}

impl fmt::Display for SdeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(reason) => {
                write!(formatter, "invalid SDE discovery configuration: {reason}")
            }
            Self::UnknownStateColumn(id) => {
                write!(formatter, "state column '{id}' is not present in the dataset")
            }
            Self::TooFewSamples { rows } => {
                write!(formatter, "need at least two rows to form an increment, got {rows}")
            }
            Self::IrregularTimeAxis => write!(
                formatter,
                "time axis is not regularly spaced within tolerance and regular spacing was required"
            ),
            Self::DegenerateState { state } => write!(
                formatter,
                "state '{state}' takes a single value; the state space cannot be binned"
            ),
            Self::TooFewPopulatedBins { state, populated, required } => write!(
                formatter,
                "state '{state}' has {populated} bins meeting min_bin_count but the library needs \
                 at least {required} for a determined sparse fit"
            ),
            Self::Feature(reason) => write!(formatter, "feature library error: {reason}"),
            Self::Sparse(error) => write!(formatter, "sparse regression error: {error}"),
            Self::Internal(reason) => write!(formatter, "internal SDE discovery error: {reason}"),
        }
    }
}

impl std::error::Error for SdeError {}

impl From<SparseError> for SdeError {
    fn from(error: SparseError) -> Self {
        Self::Sparse(error)
    }
}
