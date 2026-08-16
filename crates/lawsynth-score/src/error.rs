use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum ScoreError {
    EmptyObservations,
    LengthMismatch,
    NonFiniteValue,
    InvalidDegreesOfFreedom,
    InvalidConfig,
    InconsistentSelectionWidth,
}

impl fmt::Display for ScoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyObservations => {
                write!(formatter, "scoring requires at least one observation")
            }
            Self::LengthMismatch => write!(formatter, "observed and predicted values must align"),
            Self::NonFiniteValue => write!(formatter, "scoring values must be finite"),
            Self::InvalidDegreesOfFreedom => {
                write!(formatter, "model degrees of freedom are invalid")
            }
            Self::InvalidConfig => write!(formatter, "scoring configuration is invalid"),
            Self::InconsistentSelectionWidth => {
                write!(formatter, "selection masks must have equal width")
            }
        }
    }
}

impl std::error::Error for ScoreError {}
