use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DynamicsError {
    NoStates,
    MissingState(String),
    TooFewSamples,
    DuplicateVariable(String),
    MissingInput(String),
    StateInputOverlap(String),
    InvalidLag,
    InvalidConfig,
    NonFiniteValue,
}

impl fmt::Display for DynamicsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoStates => write!(formatter, "dynamics problem requires at least one state"),
            Self::MissingState(state) => write!(formatter, "dataset has no '{state}' state column"),
            Self::TooFewSamples => {
                write!(formatter, "dynamics problem requires at least two samples")
            }
            Self::DuplicateVariable(variable) => {
                write!(formatter, "dynamics variable '{variable}' is repeated")
            }
            Self::MissingInput(input) => write!(formatter, "dataset has no '{input}' input column"),
            Self::StateInputOverlap(variable) => {
                write!(formatter, "'{variable}' cannot be both state and input")
            }
            Self::InvalidLag => write!(
                formatter,
                "delay lag must leave at least one aligned sample"
            ),
            Self::InvalidConfig => write!(formatter, "dynamics configuration is invalid"),
            Self::NonFiniteValue => write!(formatter, "dynamics values must be finite"),
        }
    }
}

impl std::error::Error for DynamicsError {}
