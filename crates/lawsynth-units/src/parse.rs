use crate::{Unit, UnitError};

/// Parses a unit expression using the standard LawSynth built-in vocabulary.
/// For domain-specific names, use [`UnitRegistry::parse`](crate::UnitRegistry::parse).
pub fn parse_unit(expression: &str) -> Result<Unit, UnitError> {
    Unit::parse(expression)
}
