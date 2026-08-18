use std::fmt;

/// Errors returned when a cross-validated model-selection sweep cannot be run.
///
/// A candidate that *individually* fails discovery or simulation on a fold is
/// **not** an error: it is recorded as a per-fold failure in the report (see
/// [`crate::FoldStatus`]). These variants cover only conditions that make the
/// whole sweep impossible before any candidate is scored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelSelectError {
    /// The hyperparameter grid was empty, so there is nothing to select from.
    EmptyGrid,
    /// The requested fold count was zero (at least one fold is required).
    InvalidFoldCount,
    /// The dataset cannot be split into the requested folds while giving every
    /// fold at least `min_train_samples` training and `min_test_samples` test
    /// observations.
    DatasetTooShort {
        /// Total observations available on the time axis.
        samples: usize,
        /// Number of folds requested.
        folds: usize,
        /// Minimum training observations required per fold.
        min_train: usize,
        /// Minimum test observations required per fold.
        min_test: usize,
    },
    /// A fold sub-dataset could not be constructed from a contiguous slice.
    /// Carries the underlying data-layer message.
    Data(String),
}

impl fmt::Display for ModelSelectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyGrid => write!(f, "hyperparameter grid is empty"),
            Self::InvalidFoldCount => write!(f, "cross-validation requires at least one fold"),
            Self::DatasetTooShort { samples, folds, min_train, min_test } => write!(
                f,
                "dataset of {samples} samples cannot form {folds} folds with at least \
                 {min_train} train and {min_test} test samples each",
            ),
            Self::Data(message) => write!(f, "failed to build a fold sub-dataset: {message}"),
        }
    }
}

impl std::error::Error for ModelSelectError {}
