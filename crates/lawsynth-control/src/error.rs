use std::fmt;

use lawsynth_data::DataError;
use lawsynth_differentiate::DifferentiationError;
use lawsynth_features::FeatureError;
use lawsynth_sparse::SparseError;

/// Errors surfaced while building or solving a controlled (SINDYc) model.
///
/// Every variant maps a boundary failure onto a caller-actionable message. The
/// wrapped-crate variants (`Feature`, `Sparse`, `Differentiation`, `Data`)
/// preserve the underlying cause so failures remain diagnosable without leaking
/// internal types into the happy path.
#[derive(Clone, Debug, PartialEq)]
pub enum ControlError {
    /// The [`ControlSpec`](crate::ControlSpec) designated no state variables.
    NoStates,
    /// The [`ControlSpec`](crate::ControlSpec) designated no control variables.
    ///
    /// Controlled discovery requires at least one exogenous input; a run with no
    /// controls is ordinary SINDy and belongs to a different entry point.
    NoControls,
    /// An identifier appears more than once within the states or within the controls.
    DuplicateIdentifier(String),
    /// An identifier was declared as both a state and a control.
    StateControlOverlap(String),
    /// A designated state or control identifier is absent from the dataset.
    UnknownIdentifier(String),
    /// A regression target and the feature matrix disagree on row count.
    ///
    /// Under the public API this is unreachable (targets are differentiated from
    /// the same grid that produced the library rows); it exists to make the
    /// internal contract explicit and testable.
    LengthMismatch { targets: usize, rows: usize },
    /// Feature-library construction or evaluation failed.
    Feature(FeatureError),
    /// The sparse regression solve failed.
    Sparse(SparseError),
    /// Numerical differentiation of a state column failed.
    Differentiation(DifferentiationError),
    /// A derived dataset (e.g. the state-only subset) failed validation.
    Data(DataError),
}

impl fmt::Display for ControlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoStates => write!(formatter, "controlled discovery requires at least one state"),
            Self::NoControls => write!(
                formatter,
                "controlled discovery requires at least one measured control input"
            ),
            Self::DuplicateIdentifier(id) => {
                write!(formatter, "identifier '{id}' is declared more than once")
            }
            Self::StateControlOverlap(id) => {
                write!(formatter, "identifier '{id}' is declared as both a state and a control")
            }
            Self::UnknownIdentifier(id) => {
                write!(formatter, "dataset has no column '{id}' for the designated variable")
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

impl std::error::Error for ControlError {}

impl From<FeatureError> for ControlError {
    fn from(error: FeatureError) -> Self {
        Self::Feature(error)
    }
}

impl From<SparseError> for ControlError {
    fn from(error: SparseError) -> Self {
        Self::Sparse(error)
    }
}

impl From<DifferentiationError> for ControlError {
    fn from(error: DifferentiationError) -> Self {
        Self::Differentiation(error)
    }
}

impl From<DataError> for ControlError {
    fn from(error: DataError) -> Self {
        Self::Data(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_sparse_errors_through_from() {
        let error: ControlError = SparseError::SingularSystem.into();
        assert_eq!(error, ControlError::Sparse(SparseError::SingularSystem));
        assert!(error.to_string().contains("singular"));
    }

    #[test]
    fn wraps_differentiation_errors_through_from() {
        let error: ControlError = DifferentiationError::TooFewSamples.into();
        assert!(matches!(error, ControlError::Differentiation(_)));
    }

    #[test]
    fn renders_length_mismatch_message() {
        let error = ControlError::LengthMismatch { targets: 4, rows: 5 };
        assert!(error.to_string().contains("4 samples"));
        assert!(error.to_string().contains("5 rows"));
    }
}
