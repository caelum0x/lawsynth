use std::fmt;

/// Errors returned by structural-reduction detection.
///
/// `f64`-carrying variants prevent an `Eq` derive, so this type is only
/// `PartialEq`; that is sufficient for assertions in tests.
#[derive(Clone, Debug, PartialEq)]
pub enum ReduceError {
    /// The requested target column was not present in the dataset.
    UnknownTarget { target: String },
    /// The dataset had no input columns once the target was removed.
    NoInputColumns,
    /// More input variables than `max_variables` allows.
    TooManyVariables { available: usize, allowed: usize },
    /// A configuration value was non-finite or otherwise out of range.
    InvalidConfig { field: &'static str },
    /// A numerical derivative estimate failed.
    Differentiation(String),
}

impl fmt::Display for ReduceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReduceError::UnknownTarget { target } => {
                write!(formatter, "target column `{target}` is not in the dataset")
            }
            ReduceError::NoInputColumns => {
                formatter.write_str("no input columns remain after removing the target")
            }
            ReduceError::TooManyVariables { available, allowed } => write!(
                formatter,
                "structural reduction supports at most {allowed} input variables, got {available}"
            ),
            ReduceError::InvalidConfig { field } => {
                write!(formatter, "invalid reduce configuration value for `{field}`")
            }
            ReduceError::Differentiation(message) => {
                write!(formatter, "numerical differentiation failed: {message}")
            }
        }
    }
}

impl std::error::Error for ReduceError {}
