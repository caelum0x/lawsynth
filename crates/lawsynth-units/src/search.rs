//! Wildcard-aware dimensional analysis for equation-discovery search.
//!
//! [`infer_expression_dimension`](crate::infer_expression_dimension) answers a
//! stricter question — the exact dimension of a *complete* expression whose every
//! numeric literal is dimensionless. In-loop discovery needs a looser rule: free
//! numeric constants (fit coefficients, offsets, transcendental arguments) are
//! **wildcards** that may absorb whatever dimension keeps the term consistent,
//! mirroring PySR's `WildcardQuantity`. This module implements that relaxation so
//! discovery can reject dimensionally-impossible candidate terms *before* scoring
//! while never rejecting a term a dimensionful coefficient could rescue.

use std::collections::BTreeMap;

use lawsynth_core::Identifier;
use lawsynth_expr::{BinaryOperator, Expr, UnaryOperator};

use crate::{Dimension, UnitError};

/// The dimension of a subexpression under the wildcard relaxation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DimensionTerm {
    /// Not yet determined: a free numeric constant (or an undeclared symbol) that
    /// may take whatever dimension makes the enclosing term consistent.
    Wildcard,
    /// A fully determined SI dimension.
    Fixed(Dimension),
}

impl DimensionTerm {
    /// True when this term can stand in for the given target dimension: a
    /// wildcard matches anything, a fixed dimension only its equal.
    pub fn matches(self, target: Dimension) -> bool {
        match self {
            Self::Wildcard => true,
            Self::Fixed(dimension) => dimension == target,
        }
    }
}

/// Infers the wildcard-aware dimension of `expression`.
///
/// Numeric constants and symbols missing from `dimensions` are wildcards; every
/// declared symbol carries its fixed dimension. Returns `Err` only for a term
/// that is dimensionally impossible for *any* assignment of the wildcards (e.g.
/// adding a length to a velocity, or taking `sin` of a dimensioned quantity).
pub fn infer_term_dimension(
    expression: &Expr,
    dimensions: &BTreeMap<Identifier, Dimension>,
) -> Result<DimensionTerm, UnitError> {
    match expression {
        Expr::Constant(_) => Ok(DimensionTerm::Wildcard),
        Expr::Symbol(id) => {
            Ok(dimensions.get(id).copied().map_or(DimensionTerm::Wildcard, DimensionTerm::Fixed))
        }
        Expr::Unary { operator, operand } => {
            let operand = infer_term_dimension(operand, dimensions)?;
            match operator {
                UnaryOperator::Negate => Ok(operand),
                UnaryOperator::Exp
                | UnaryOperator::Log
                | UnaryOperator::Sin
                | UnaryOperator::Cos => match operand {
                    DimensionTerm::Wildcard => Ok(DimensionTerm::Fixed(Dimension::DIMENSIONLESS)),
                    DimensionTerm::Fixed(Dimension::DIMENSIONLESS) => {
                        Ok(DimensionTerm::Fixed(Dimension::DIMENSIONLESS))
                    }
                    DimensionTerm::Fixed(_) => Err(UnitError::IncompatibleDimensions),
                },
            }
        }
        Expr::Binary { operator, left, right } => {
            let left_dimension = infer_term_dimension(left, dimensions)?;
            let right_dimension = infer_term_dimension(right, dimensions)?;
            match operator {
                BinaryOperator::Add | BinaryOperator::Subtract => {
                    combine_additive(left_dimension, right_dimension)
                }
                BinaryOperator::Multiply => combine_scaled(left_dimension, right_dimension, false),
                BinaryOperator::Divide => combine_scaled(left_dimension, right_dimension, true),
                BinaryOperator::Power => infer_power(left_dimension, right_dimension, right),
            }
        }
    }
}

/// True when `expression` is dimensionally consistent with `target`: a wildcard
/// result matches any target, a fixed result only its equal, and an impossible
/// term never matches.
pub fn admits_dimension(
    expression: &Expr,
    dimensions: &BTreeMap<Identifier, Dimension>,
    target: Dimension,
) -> bool {
    matches!(infer_term_dimension(expression, dimensions), Ok(term) if term.matches(target))
}

/// True when a free multiplicative coefficient could rescale `expression` to
/// `target`.
///
/// In a SINDy-style law each library term is fitted with its own numeric
/// coefficient, itself a wildcard constant. Prefixing that wildcard coefficient
/// means any *internally consistent* term can reach `target`; only a
/// dimensionally impossible term (inference error) is rejected. This is the
/// predicate discovery applies to candidate feature terms.
pub fn admits_scaled_dimension(
    expression: &Expr,
    dimensions: &BTreeMap<Identifier, Dimension>,
    target: Dimension,
) -> bool {
    let scaled = Expr::product(Expr::constant(1.0), expression.clone());
    admits_dimension(&scaled, dimensions, target)
}

fn combine_additive(left: DimensionTerm, right: DimensionTerm) -> Result<DimensionTerm, UnitError> {
    match (left, right) {
        (DimensionTerm::Wildcard, other) | (other, DimensionTerm::Wildcard) => Ok(other),
        (DimensionTerm::Fixed(left), DimensionTerm::Fixed(right)) if left == right => {
            Ok(DimensionTerm::Fixed(left))
        }
        (DimensionTerm::Fixed(_), DimensionTerm::Fixed(_)) => {
            Err(UnitError::IncompatibleDimensions)
        }
    }
}

fn combine_scaled(
    left: DimensionTerm,
    right: DimensionTerm,
    divide: bool,
) -> Result<DimensionTerm, UnitError> {
    match (left, right) {
        (DimensionTerm::Fixed(left), DimensionTerm::Fixed(right)) => {
            let combined = if divide { left.divide(right) } else { left.multiply(right) };
            combined.map(DimensionTerm::Fixed).ok_or(UnitError::DimensionOverflow)
        }
        _ => Ok(DimensionTerm::Wildcard),
    }
}

fn infer_power(
    base: DimensionTerm,
    exponent: DimensionTerm,
    exponent_expression: &Expr,
) -> Result<DimensionTerm, UnitError> {
    // A dimensioned exponent is never physical.
    if let DimensionTerm::Fixed(dimension) = exponent {
        if dimension != Dimension::DIMENSIONLESS {
            return Err(UnitError::IncompatibleDimensions);
        }
    }
    match base {
        DimensionTerm::Wildcard => Ok(DimensionTerm::Wildcard),
        DimensionTerm::Fixed(Dimension::DIMENSIONLESS) => {
            Ok(DimensionTerm::Fixed(Dimension::DIMENSIONLESS))
        }
        DimensionTerm::Fixed(dimension) => match exponent_expression {
            Expr::Constant(value)
                if value.fract() == 0.0 && *value >= i8::MIN as f64 && *value <= i8::MAX as f64 =>
            {
                dimension
                    .pow(*value as i8)
                    .map(DimensionTerm::Fixed)
                    .ok_or(UnitError::DimensionOverflow)
            }
            _ => Err(UnitError::IncompatibleDimensions),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Unit;

    fn id(name: &str) -> Identifier {
        Identifier::new(name).unwrap()
    }

    fn oscillator_dimensions() -> BTreeMap<Identifier, Dimension> {
        BTreeMap::from([
            (id("x"), Unit::parse("m").unwrap().dimension()),
            (id("v"), Unit::parse("m/s").unwrap().dimension()),
        ])
    }

    fn acceleration() -> Dimension {
        Unit::parse("m/s^2").unwrap().dimension()
    }

    #[test]
    fn a_bare_symbol_carries_its_declared_dimension() {
        let dimensions = oscillator_dimensions();
        assert_eq!(
            infer_term_dimension(&Expr::symbol(id("x")), &dimensions).unwrap(),
            DimensionTerm::Fixed(Unit::parse("m").unwrap().dimension())
        );
    }

    #[test]
    fn a_free_constant_is_a_wildcard() {
        let dimensions = oscillator_dimensions();
        assert_eq!(
            infer_term_dimension(&Expr::constant(2.5), &dimensions).unwrap(),
            DimensionTerm::Wildcard
        );
    }

    #[test]
    fn sine_of_a_dimensioned_argument_is_rejected() {
        let dimensions = oscillator_dimensions();
        let sine = Expr::unary(UnaryOperator::Sin, Expr::symbol(id("x")));
        assert!(infer_term_dimension(&sine, &dimensions).is_err());
        assert!(!admits_scaled_dimension(&sine, &dimensions, acceleration()));
    }

    #[test]
    fn adding_a_length_to_a_velocity_is_rejected() {
        let dimensions = oscillator_dimensions();
        let mixed = Expr::sum(Expr::symbol(id("x")), Expr::symbol(id("v")));
        assert!(infer_term_dimension(&mixed, &dimensions).is_err());
    }

    #[test]
    fn a_wildcard_constant_absorbs_an_addition() {
        let dimensions = oscillator_dimensions();
        // x + 3  ->  the free constant 3 becomes a length so the sum is a length.
        let shifted = Expr::sum(Expr::symbol(id("x")), Expr::constant(3.0));
        assert_eq!(
            infer_term_dimension(&shifted, &dimensions).unwrap(),
            DimensionTerm::Fixed(Unit::parse("m").unwrap().dimension())
        );
    }

    #[test]
    fn a_scaled_polynomial_term_reaches_any_target() {
        let dimensions = oscillator_dimensions();
        // A fit coefficient can rescale x (a length) to the acceleration target.
        assert!(admits_scaled_dimension(&Expr::symbol(id("x")), &dimensions, acceleration()));
        assert!(admits_scaled_dimension(&Expr::symbol(id("v")), &dimensions, acceleration()));
        // ... and the same holds for a quadratic interaction.
        let product = Expr::product(Expr::symbol(id("x")), Expr::symbol(id("v")));
        assert!(admits_scaled_dimension(&product, &dimensions, acceleration()));
    }

    #[test]
    fn strict_matching_respects_the_target_dimension() {
        let dimensions = oscillator_dimensions();
        // Without a rescaling coefficient, x (length) does not match acceleration.
        assert!(!admits_dimension(&Expr::symbol(id("x")), &dimensions, acceleration()));
        // But it does match the length target.
        assert!(admits_dimension(
            &Expr::symbol(id("x")),
            &dimensions,
            Unit::parse("m").unwrap().dimension()
        ));
    }

    #[test]
    fn undeclared_symbols_stay_wildcards_so_nothing_is_pruned() {
        let dimensions = BTreeMap::new();
        let expression = Expr::sum(Expr::symbol(id("x")), Expr::symbol(id("v")));
        assert_eq!(
            infer_term_dimension(&expression, &dimensions).unwrap(),
            DimensionTerm::Wildcard
        );
        assert!(admits_scaled_dimension(&expression, &dimensions, acceleration()));
    }
}
