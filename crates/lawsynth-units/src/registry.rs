use std::collections::BTreeMap;

use crate::{Unit, UnitError};

/// Deterministic registry for built-in and domain-specific scaled units.
#[derive(Clone, Debug, PartialEq)]
pub struct UnitRegistry {
    units: BTreeMap<String, Unit>,
}

impl Default for UnitRegistry {
    fn default() -> Self {
        let mut registry = Self { units: BTreeMap::new() };
        for name in ["1", "m", "km", "s", "min", "kg", "g"] {
            registry
                .units
                .insert(name.to_owned(), Unit::parse(name).expect("built-in unit must parse"));
        }
        registry
    }
}

impl UnitRegistry {
    pub fn register(&mut self, name: impl Into<String>, unit: Unit) -> Result<(), UnitError> {
        let name = name.into();
        if name.is_empty() {
            return Err(UnitError::InvalidExpression);
        }
        if self.units.contains_key(&name) {
            return Err(UnitError::DuplicateUnit(name));
        }
        self.units.insert(name, unit);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Unit> {
        self.units.get(name)
    }

    /// Parses multiplication, division, and integer powers using registered names.
    pub fn parse(&self, expression: &str) -> Result<Unit, UnitError> {
        if expression.is_empty() {
            return Err(UnitError::InvalidExpression);
        }
        let mut result = Unit::dimensionless();
        let mut start = 0;
        let mut divide = false;
        for (index, character) in
            expression.char_indices().chain(std::iter::once((expression.len(), '*')))
        {
            if character != '*' && character != '/' {
                continue;
            }
            let factor = self.factor(&expression[start..index])?;
            result = if divide { result.divide(&factor)? } else { result.multiply(&factor)? };
            divide = character == '/';
            start = index + character.len_utf8();
        }
        Ok(result)
    }

    fn factor(&self, value: &str) -> Result<Unit, UnitError> {
        let (name, exponent) = match value.split_once('^') {
            Some((name, exponent)) => {
                (name, exponent.parse().map_err(|_| UnitError::ExponentOutOfRange)?)
            }
            None => (value, 1_i8),
        };
        self.get(name)
            .cloned()
            .ok_or_else(|| UnitError::UnknownUnit(name.to_owned()))?
            .pow(exponent)
    }
}

#[cfg(test)]
mod tests {
    use crate::Dimension;

    use super::*;

    #[test]
    fn parses_custom_composite_units() {
        let mut registry = UnitRegistry::default();
        registry
            .register("ft", Unit::from_parts("ft", Dimension::LENGTH, 0.3048).unwrap())
            .unwrap();
        let velocity = registry.parse("ft/min").unwrap();
        assert!(velocity.compatible_with(&Unit::parse("m/s").unwrap()));
        assert!((velocity.scale_to_si() - 0.00508).abs() < 1e-12);
    }
}
