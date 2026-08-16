use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProfileError {
    EmptyColumn,
    NonFiniteValues,
    LengthMismatch,
    TooFewValues,
    ConstantValues,
    InvalidConfiguration,
}

impl fmt::Display for ProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyColumn => write!(formatter, "cannot profile an empty column"),
            Self::NonFiniteValues => write!(formatter, "profile values must be finite"),
            Self::LengthMismatch => write!(formatter, "profile inputs must have equal lengths"),
            Self::TooFewValues => write!(formatter, "at least two values are required"),
            Self::ConstantValues => {
                write!(formatter, "correlation is undefined for constant values")
            }
            Self::InvalidConfiguration => write!(formatter, "profile configuration is invalid"),
        }
    }
}

impl std::error::Error for ProfileError {}
