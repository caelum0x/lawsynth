use std::fmt;

/// Errors returned by the implicit / rational discovery boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImplicitError {
    /// The [`ImplicitConfig`](crate::ImplicitConfig) held an invalid field
    /// (non-finite threshold, zero degree, empty iteration budget, ...).
    InvalidConfig,
    /// The requested target column is absent from the dataset.
    UnknownTarget(String),
    /// The dataset had too few samples to estimate a derivative and fit a
    /// relation after trimming the (least accurate) boundary rows.
    InsufficientSamples,
    /// Derivative estimation of the target state failed.
    Differentiation(String),
    /// Every candidate left-hand side produced a degenerate or singular fit, so
    /// no non-trivial implicit relation could be normalised.
    NoRelation,
    /// A computed value was not finite.
    NonFiniteValue,
}

impl fmt::Display for ImplicitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig => write!(formatter, "implicit discovery configuration is invalid"),
            Self::UnknownTarget(name) => {
                write!(formatter, "target column `{name}` is not in the dataset")
            }
            Self::InsufficientSamples => {
                write!(formatter, "not enough samples to form an implicit relation")
            }
            Self::Differentiation(reason) => {
                write!(formatter, "derivative estimation failed: {reason}")
            }
            Self::NoRelation => {
                write!(formatter, "no non-trivial implicit relation could be found")
            }
            Self::NonFiniteValue => write!(formatter, "computed values must be finite"),
        }
    }
}

impl std::error::Error for ImplicitError {}
