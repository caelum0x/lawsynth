use std::collections::BTreeMap;

use lawsynth_core::Identifier;
use lawsynth_expr::{BinaryOperator, Expr, UnaryOperator};

use crate::{Dimension, UnitError, unit::Unit};

/// Infers the dimension of a scalar expression from a complete symbol-unit map.
pub fn infer_expression_dimension(
    expression: &Expr,
    symbols: &BTreeMap<Identifier, Unit>,
) -> Result<Dimension, UnitError> {
    match expression {
        Expr::Constant(_) => Ok(Dimension::DIMENSIONLESS),
        Expr::Symbol(id) => symbols
            .get(id)
            .map(Unit::dimension)
            .ok_or_else(|| UnitError::UnknownSymbol(id.to_string())),
        Expr::Unary { operator, operand } => {
            let dimension = infer_expression_dimension(operand, symbols)?;
            match operator {
                UnaryOperator::Negate => Ok(dimension),
                UnaryOperator::Exp
                | UnaryOperator::Log
                | UnaryOperator::Sin
                | UnaryOperator::Cos
                    if dimension == Dimension::DIMENSIONLESS =>
                {
                    Ok(Dimension::DIMENSIONLESS)
                }
                UnaryOperator::Exp
                | UnaryOperator::Log
                | UnaryOperator::Sin
                | UnaryOperator::Cos => Err(UnitError::IncompatibleDimensions),
            }
        }
        Expr::Binary { operator, left, right } => {
            let left_dimension = infer_expression_dimension(left, symbols)?;
            let right_dimension = infer_expression_dimension(right, symbols)?;
            match operator {
                BinaryOperator::Add | BinaryOperator::Subtract
                    if left_dimension == right_dimension =>
                {
                    Ok(left_dimension)
                }
                BinaryOperator::Add | BinaryOperator::Subtract => {
                    Err(UnitError::IncompatibleDimensions)
                }
                BinaryOperator::Multiply => {
                    left_dimension.multiply(right_dimension).ok_or(UnitError::DimensionOverflow)
                }
                BinaryOperator::Divide => {
                    left_dimension.divide(right_dimension).ok_or(UnitError::DimensionOverflow)
                }
                BinaryOperator::Power if left_dimension == Dimension::DIMENSIONLESS => {
                    Ok(Dimension::DIMENSIONLESS)
                }
                BinaryOperator::Power => match right.as_ref() {
                    Expr::Constant(value)
                        if value.fract() == 0.0
                            && *value >= i8::MIN as f64
                            && *value <= i8::MAX as f64 =>
                    {
                        left_dimension.pow(*value as i8).ok_or(UnitError::DimensionOverflow)
                    }
                    _ => Err(UnitError::IncompatibleDimensions),
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use lawsynth_core::Identifier;
    use lawsynth_expr::Expr;

    use crate::{Unit, infer_expression_dimension};

    #[test]
    fn infers_acceleration_from_velocity_over_time() {
        let id = |value| Identifier::new(value).unwrap();
        let expression = Expr::quotient(Expr::symbol(id("velocity")), Expr::symbol(id("time")));
        let units = BTreeMap::from([
            (id("velocity"), Unit::parse("m/s").unwrap()),
            (id("time"), Unit::parse("s").unwrap()),
        ]);
        assert_eq!(
            infer_expression_dimension(&expression, &units).unwrap(),
            Unit::parse("m/s^2").unwrap().dimension()
        );
    }
}
