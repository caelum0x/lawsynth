use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnitError {
    UnknownUnit(String),
    UnknownSymbol(String),
    InvalidExpression,
    ExponentOutOfRange,
    DimensionOverflow,
    IncompatibleDimensions,
    NonFiniteValue,
    InvalidScale,
    DuplicateUnit(String),
}

impl fmt::Display for UnitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownUnit(unit) => write!(formatter, "unknown unit '{unit}'"),
            Self::UnknownSymbol(symbol) => {
                write!(formatter, "no unit is declared for symbol '{symbol}'")
            }
            Self::InvalidExpression => write!(formatter, "invalid unit expression"),
            Self::ExponentOutOfRange => {
                write!(formatter, "unit exponent is outside the supported range")
            }
            Self::DimensionOverflow => write!(formatter, "unit dimension exponent overflow"),
            Self::IncompatibleDimensions => write!(formatter, "units have incompatible dimensions"),
            Self::NonFiniteValue => write!(formatter, "unit conversion requires a finite value"),
            Self::InvalidScale => write!(formatter, "unit scale must be finite and positive"),
            Self::DuplicateUnit(unit) => write!(formatter, "unit '{unit}' is already registered"),
        }
    }
}

impl std::error::Error for UnitError {}
