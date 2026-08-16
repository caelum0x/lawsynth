use crate::UnitRegistry;

/// Returns a fresh registry containing SI-scaled built-in units.
pub fn builtin_registry() -> UnitRegistry {
    UnitRegistry::default()
}
