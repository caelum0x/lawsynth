use crate::{Dimension, UnitError};

/// A scaled SI unit with an inspectable canonical expression.
#[derive(Clone, Debug, PartialEq)]
pub struct Unit {
    canonical: String,
    dimension: Dimension,
    scale: f64,
}

impl Unit {
    /// Creates a named unit from an SI dimension and scale multiplier.
    pub fn from_parts(
        canonical: impl Into<String>,
        dimension: Dimension,
        scale_to_si: f64,
    ) -> Result<Self, UnitError> {
        let canonical = canonical.into();
        if canonical.is_empty() {
            return Err(UnitError::InvalidExpression);
        }
        if !scale_to_si.is_finite() || scale_to_si <= 0.0 {
            return Err(UnitError::InvalidScale);
        }
        Ok(Self {
            canonical,
            dimension,
            scale: scale_to_si,
        })
    }

    pub fn dimensionless() -> Self {
        Self {
            canonical: "1".to_owned(),
            dimension: Dimension::DIMENSIONLESS,
            scale: 1.0,
        }
    }

    pub fn parse(expression: &str) -> Result<Self, UnitError> {
        if expression.is_empty() {
            return Err(UnitError::InvalidExpression);
        }
        let mut result = Self::dimensionless();
        let mut start = 0;
        let mut divide = false;
        for (index, character) in expression
            .char_indices()
            .chain(std::iter::once((expression.len(), '*')))
        {
            if character != '*' && character != '/' {
                continue;
            }
            let factor = parse_factor(&expression[start..index])?;
            result = if divide {
                result.divide(&factor)?
            } else {
                result.multiply(&factor)?
            };
            divide = character == '/';
            start = index + character.len_utf8();
        }
        Ok(Self {
            canonical: expression.to_owned(),
            ..result
        })
    }

    pub fn canonical(&self) -> &str {
        &self.canonical
    }

    pub fn dimension(&self) -> Dimension {
        self.dimension
    }

    pub fn compatible_with(&self, other: &Self) -> bool {
        self.dimension == other.dimension
    }

    /// Multiplicative scale that maps this unit to coherent SI base units.
    pub fn scale_to_si(&self) -> f64 {
        self.scale
    }

    pub fn multiply(&self, other: &Self) -> Result<Self, UnitError> {
        Ok(Self {
            canonical: format!("{}*{}", self.canonical, other.canonical),
            dimension: self
                .dimension
                .multiply(other.dimension)
                .ok_or(UnitError::DimensionOverflow)?,
            scale: self.scale * other.scale,
        })
    }

    pub fn divide(&self, other: &Self) -> Result<Self, UnitError> {
        Ok(Self {
            canonical: format!("{}/{}", self.canonical, other.canonical),
            dimension: self
                .dimension
                .divide(other.dimension)
                .ok_or(UnitError::DimensionOverflow)?,
            scale: self.scale / other.scale,
        })
    }

    /// Raises a unit to an integer exponent while preserving its scale.
    pub fn pow(&self, exponent: i8) -> Result<Self, UnitError> {
        Ok(Self {
            canonical: format!("{}^{exponent}", self.canonical),
            dimension: self
                .dimension
                .pow(exponent)
                .ok_or(UnitError::DimensionOverflow)?,
            scale: self.scale.powi(i32::from(exponent)),
        })
    }
}

fn parse_factor(value: &str) -> Result<Unit, UnitError> {
    let (name, exponent) = match value.split_once('^') {
        Some((name, exponent)) => (
            name,
            exponent
                .parse()
                .map_err(|_| UnitError::ExponentOutOfRange)?,
        ),
        None => (value, 1_i8),
    };
    let mut factor = named(name)?;
    factor.dimension = factor
        .dimension
        .pow(exponent)
        .ok_or(UnitError::DimensionOverflow)?;
    factor.scale = factor.scale.powi(i32::from(exponent));
    Ok(factor)
}

fn named(name: &str) -> Result<Unit, UnitError> {
    let (dimension, scale) = match name {
        "1" => (Dimension::DIMENSIONLESS, 1.0),
        "m" => (Dimension::LENGTH, 1.0),
        "km" => (Dimension::LENGTH, 1_000.0),
        "s" => (Dimension::TIME, 1.0),
        "min" => (Dimension::TIME, 60.0),
        "kg" => (Dimension::MASS, 1.0),
        "g" => (Dimension::MASS, 0.001),
        _ => return Err(UnitError::UnknownUnit(name.to_owned())),
    };
    Ok(Unit {
        canonical: name.to_owned(),
        dimension,
        scale,
    })
}

#[cfg(test)]
mod tests {
    use super::Unit;

    #[test]
    fn parses_a_velocity_unit() {
        let velocity = Unit::parse("m/s").unwrap();
        assert!(velocity.compatible_with(&Unit::parse("km/min").unwrap()));
    }
}
