use crate::{Unit, UnitError};

/// Converts a finite scalar between dimensionally compatible units.
pub fn convert(value: f64, from: &Unit, to: &Unit) -> Result<f64, UnitError> {
    if !value.is_finite() {
        return Err(UnitError::NonFiniteValue);
    }
    if !from.compatible_with(to) {
        return Err(UnitError::IncompatibleDimensions);
    }
    let converted = value * from.scale_to_si() / to.scale_to_si();
    if converted.is_finite() {
        Ok(converted)
    } else {
        Err(UnitError::NonFiniteValue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_scaled_compatible_units() {
        assert_eq!(
            convert(1.5, &Unit::parse("km").unwrap(), &Unit::parse("m").unwrap()).unwrap(),
            1_500.0
        );
        assert_eq!(
            convert(
                120.0,
                &Unit::parse("s").unwrap(),
                &Unit::parse("min").unwrap()
            )
            .unwrap(),
            2.0
        );
    }

    #[test]
    fn rejects_incompatible_units() {
        assert_eq!(
            convert(1.0, &Unit::parse("m").unwrap(), &Unit::parse("s").unwrap()),
            Err(UnitError::IncompatibleDimensions)
        );
    }
}
