use crate::{Unit, UnitError};

/// Requires two units to represent the same physical dimension.
pub fn require_compatible(actual: &Unit, expected: &Unit) -> Result<(), UnitError> {
    actual.compatible_with(expected).then_some(()).ok_or(UnitError::IncompatibleDimensions)
}
