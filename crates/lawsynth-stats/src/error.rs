use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatsError {
    EmptyInput,
    InvalidBootstrapConfig,
    InvalidConfidence,
    InvalidProbability,
    LengthMismatch,
    TooFewValues,
    NonFiniteValue,
    ConstantValues,
    InvalidStandardDeviation,
    InvalidHistogramConfig,
    SampleExceedsPopulation,
}

impl fmt::Display for StatsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(formatter, "statistical input cannot be empty"),
            Self::InvalidBootstrapConfig => write!(formatter, "invalid bootstrap configuration"),
            Self::InvalidConfidence => write!(formatter, "confidence must be between zero and one"),
            Self::InvalidProbability => write!(
                formatter,
                "probability must be finite and between zero and one"
            ),
            Self::LengthMismatch => write!(formatter, "statistical inputs must have equal lengths"),
            Self::TooFewValues => write!(formatter, "at least two values are required"),
            Self::NonFiniteValue => write!(formatter, "statistical inputs must be finite"),
            Self::ConstantValues => write!(formatter, "statistic is undefined for constant values"),
            Self::InvalidStandardDeviation => {
                write!(formatter, "standard deviation must be finite and positive")
            }
            Self::InvalidHistogramConfig => {
                write!(formatter, "histogram bin count must be positive")
            }
            Self::SampleExceedsPopulation => {
                write!(formatter, "sample size exceeds population size")
            }
        }
    }
}

impl std::error::Error for StatsError {}
