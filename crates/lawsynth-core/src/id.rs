use std::{fmt, str::FromStr};

use crate::IdentifierError;

/// A portable symbol identifier used throughout World IR.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Identifier(String);

impl Identifier {
    pub fn new(value: impl Into<String>) -> Result<Self, IdentifierError> {
        let value = value.into();
        if value.is_empty() {
            return Err(IdentifierError::Empty);
        }
        if value.as_bytes()[0].is_ascii_digit() {
            return Err(IdentifierError::StartsWithDigit);
        }
        for (index, character) in value.char_indices() {
            if !(character.is_ascii_alphanumeric() || character == '_' || character == '-') {
                return Err(IdentifierError::InvalidCharacter { character, index });
            }
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FromStr for Identifier {
    type Err = IdentifierError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_portable_identifiers() {
        assert_eq!(
            Identifier::new("supply-demand_2").unwrap().as_str(),
            "supply-demand_2"
        );
    }

    #[test]
    fn rejects_ambiguous_identifiers() {
        assert!(matches!(Identifier::new(""), Err(IdentifierError::Empty)));
        assert!(matches!(
            Identifier::new("2fast"),
            Err(IdentifierError::StartsWithDigit)
        ));
        assert!(matches!(
            Identifier::new("not valid"),
            Err(IdentifierError::InvalidCharacter { .. })
        ));
    }
}
