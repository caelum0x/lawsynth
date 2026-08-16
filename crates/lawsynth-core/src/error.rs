use std::fmt;

/// A reason an [`crate::Identifier`] could not be constructed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IdentifierError {
    Empty,
    InvalidCharacter { character: char, index: usize },
    StartsWithDigit,
}

impl fmt::Display for IdentifierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(formatter, "identifiers cannot be empty"),
            Self::InvalidCharacter { character, index } => write!(
                formatter,
                "invalid identifier character '{character}' at byte {index}; use ASCII letters, digits, '_' or '-'"
            ),
            Self::StartsWithDigit => write!(formatter, "identifiers cannot start with a digit"),
        }
    }
}

impl std::error::Error for IdentifierError {}
