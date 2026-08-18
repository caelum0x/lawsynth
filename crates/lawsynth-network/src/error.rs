use std::fmt;

use lawsynth_data::DataError;
use lawsynth_differentiate::DifferentiationError;
use lawsynth_features::FeatureError;
use lawsynth_sparse::SparseError;

/// Errors surfaced while discovering the coupling structure of a network.
///
/// The wrapped-crate variants (`Feature`, `Sparse`, `Differentiation`, `Data`)
/// preserve the underlying cause so failures stay diagnosable without leaking
/// internal types into the happy path.
#[derive(Clone, Debug, PartialEq)]
pub enum NetworkError {
    /// The dataset has fewer than two nodes.
    ///
    /// Coupling discovery is about which node influences which, so it needs at
    /// least two candidate nodes. A one-column dataset is ordinary single-series
    /// dynamics discovery and belongs to a different entry point.
    SingleNode(usize),
    /// The configured [`edge_threshold`](crate::NetworkConfig::edge_threshold) is
    /// not a finite, non-negative number.
    InvalidThreshold(f64),
    /// A regression target and the feature matrix disagree on row count.
    ///
    /// Under the public API this is unreachable (targets are differentiated from
    /// the same grid that produced the library rows); it exists to make the
    /// internal contract explicit and testable.
    LengthMismatch { targets: usize, rows: usize },
    /// Per-node candidate library construction or evaluation failed.
    Feature(FeatureError),
    /// A per-node sparse regression solve failed.
    Sparse(SparseError),
    /// Numerical differentiation of a node column failed.
    Differentiation(DifferentiationError),
    /// A derived dataset failed validation.
    Data(DataError),
}

impl fmt::Display for NetworkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SingleNode(count) => write!(
                formatter,
                "coupling discovery needs at least two nodes but the dataset has {count}"
            ),
            Self::InvalidThreshold(value) => {
                write!(formatter, "edge threshold must be finite and non-negative, got {value}")
            }
            Self::LengthMismatch { targets, rows } => write!(
                formatter,
                "derivative target has {targets} samples but the library has {rows} rows"
            ),
            Self::Feature(error) => write!(formatter, "feature library error: {error}"),
            Self::Sparse(error) => write!(formatter, "sparse regression error: {error}"),
            Self::Differentiation(error) => write!(formatter, "differentiation error: {error}"),
            Self::Data(error) => write!(formatter, "derived dataset error: {error}"),
        }
    }
}

impl std::error::Error for NetworkError {}

impl From<FeatureError> for NetworkError {
    fn from(error: FeatureError) -> Self {
        Self::Feature(error)
    }
}

impl From<SparseError> for NetworkError {
    fn from(error: SparseError) -> Self {
        Self::Sparse(error)
    }
}

impl From<DifferentiationError> for NetworkError {
    fn from(error: DifferentiationError) -> Self {
        Self::Differentiation(error)
    }
}

impl From<DataError> for NetworkError {
    fn from(error: DataError) -> Self {
        Self::Data(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_sparse_errors_through_from() {
        let error: NetworkError = SparseError::SingularSystem.into();
        assert_eq!(error, NetworkError::Sparse(SparseError::SingularSystem));
        assert!(error.to_string().contains("singular"));
    }

    #[test]
    fn wraps_differentiation_errors_through_from() {
        let error: NetworkError = DifferentiationError::TooFewSamples.into();
        assert!(matches!(error, NetworkError::Differentiation(_)));
    }

    #[test]
    fn renders_single_node_and_length_messages() {
        assert!(NetworkError::SingleNode(1).to_string().contains("at least two nodes"));
        let error = NetworkError::LengthMismatch { targets: 4, rows: 5 };
        assert!(error.to_string().contains("4 samples"));
        assert!(error.to_string().contains("5 rows"));
    }
}
