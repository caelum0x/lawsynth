//! Errors returned while building public API values.

use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApiValidationError {
    Empty {
        field: &'static str,
    },
    TooLong {
        field: &'static str,
        maximum: usize,
    },
    Invalid {
        field: &'static str,
        reason: &'static str,
    },
    OutOfRange {
        field: &'static str,
        minimum: u64,
        maximum: u64,
    },
    Inconsistent {
        reason: &'static str,
    },
}

impl fmt::Display for ApiValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty { field } => write!(formatter, "{field} must not be empty"),
            Self::TooLong { field, maximum } => {
                write!(formatter, "{field} exceeds {maximum} bytes")
            }
            Self::Invalid { field, reason } => write!(formatter, "invalid {field}: {reason}"),
            Self::OutOfRange {
                field,
                minimum,
                maximum,
            } => {
                write!(formatter, "{field} must be in {minimum}..={maximum}")
            }
            Self::Inconsistent { reason } => write!(formatter, "inconsistent API value: {reason}"),
        }
    }
}

impl std::error::Error for ApiValidationError {}
