use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum FeatureError {
    EmptyVariables,
    DuplicateVariable(String),
    EmptySeries,
    InvalidDelay { lag: usize, length: usize },
    MissingValue(String),
    Evaluation(String),
}

impl fmt::Display for FeatureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyVariables => {
                write!(formatter, "feature library requires at least one variable")
            }
            Self::DuplicateVariable(variable) => {
                write!(formatter, "feature library repeats variable '{variable}'")
            }
            Self::EmptySeries => write!(formatter, "delayed features require a non-empty series"),
            Self::InvalidDelay { lag, length } => write!(
                formatter,
                "delay {lag} is invalid for a series with {length} observations"
            ),
            Self::MissingValue(variable) => {
                write!(formatter, "dataset has no '{variable}' feature column")
            }
            Self::Evaluation(error) => write!(formatter, "feature evaluation failed: {error}"),
        }
    }
}

impl std::error::Error for FeatureError {}
