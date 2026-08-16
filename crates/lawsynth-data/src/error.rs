use std::fmt;

use lawsynth_core::Identifier;

#[derive(Clone, Debug, PartialEq)]
pub enum DataError {
    EmptyTimeAxis,
    NonFiniteTimestamp { index: usize, value: f64 },
    NonIncreasingTimestamp { index: usize },
    NoColumns,
    DuplicateColumn(Identifier),
    ColumnLengthMismatch { column: Identifier, expected: usize, actual: usize },
    NonFiniteValue { column: Identifier, index: usize, value: f64 },
    InvalidBatchSize,
    InvalidWindowConfig,
    Delimited(String),
    Parquet(String),
}

impl fmt::Display for DataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTimeAxis => write!(formatter, "time axis cannot be empty"),
            Self::NonFiniteTimestamp { index, value } => {
                write!(formatter, "timestamp at index {index} must be finite, got {value}")
            }
            Self::NonIncreasingTimestamp { index } => {
                write!(
                    formatter,
                    "timestamp at index {index} must be strictly greater than its predecessor"
                )
            }
            Self::NoColumns => {
                write!(formatter, "dataset must contain at least one numeric column")
            }
            Self::DuplicateColumn(id) => {
                write!(formatter, "column '{id}' is declared more than once")
            }
            Self::ColumnLengthMismatch { column, expected, actual } => write!(
                formatter,
                "column '{column}' has {actual} rows; expected {expected} to match the time axis"
            ),
            Self::NonFiniteValue { column, index, value } => {
                write!(
                    formatter,
                    "column '{column}' value at index {index} must be finite, got {value}"
                )
            }
            Self::InvalidBatchSize => write!(formatter, "batch size must be greater than zero"),
            Self::InvalidWindowConfig => {
                write!(formatter, "window width and step must be positive and fit the dataset")
            }
            Self::Delimited(reason) => {
                write!(formatter, "delimited data decoding failed: {reason}")
            }
            Self::Parquet(reason) => write!(formatter, "Parquet decoding failed: {reason}"),
        }
    }
}

impl std::error::Error for DataError {}
