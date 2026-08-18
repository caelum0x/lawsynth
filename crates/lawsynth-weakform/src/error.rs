use std::fmt;

/// Errors returned by weak / integral-form discovery.
///
/// `f64`-carrying variants prevent an `Eq` derive, so this type is only
/// `PartialEq`; that is sufficient for assertions in tests.
#[derive(Clone, Debug, PartialEq)]
pub enum WeakError {
    /// The dataset had fewer samples than weak assembly requires.
    TooFewSamples { available: usize, required: usize },
    /// `test_function_count` was zero.
    NoTestFunctions,
    /// `test_function_order` was below the smoothness floor (`p >= 2`).
    OrderTooLow { order: usize },
    /// `support_fraction` was outside the open interval `(0, 1)`.
    InvalidSupportFraction { value: f64 },
    /// A non-finite or non-positive numeric configuration value.
    InvalidConfig { field: &'static str },
    /// A test-function support window covered fewer than two samples.
    EmptySupport { center: f64, radius: f64 },
    /// The candidate feature library could not be built or evaluated.
    Feature(String),
    /// A weak normal-equations system was singular for some state.
    SingularSystem,
}

impl fmt::Display for WeakError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WeakError::TooFewSamples { available, required } => write!(
                formatter,
                "weak-form discovery needs at least {required} samples, got {available}"
            ),
            WeakError::NoTestFunctions => {
                formatter.write_str("test_function_count must be at least 1")
            }
            WeakError::OrderTooLow { order } => write!(
                formatter,
                "test_function_order must be at least 2 for a vanishing boundary, got {order}"
            ),
            WeakError::InvalidSupportFraction { value } => write!(
                formatter,
                "support_fraction must lie in the open interval (0, 1), got {value}"
            ),
            WeakError::InvalidConfig { field } => {
                write!(formatter, "invalid configuration value for `{field}`")
            }
            WeakError::EmptySupport { center, radius } => write!(
                formatter,
                "test-function support [{lo}, {hi}] contained fewer than two samples",
                lo = center - radius,
                hi = center + radius
            ),
            WeakError::Feature(message) => write!(formatter, "feature library error: {message}"),
            WeakError::SingularSystem => {
                formatter.write_str("weak normal-equations system was singular")
            }
        }
    }
}

impl std::error::Error for WeakError {}
