use std::collections::BTreeMap;

use lawsynth_core::Identifier;

use crate::{BinaryOperator, EvaluationError, Expr, UnaryOperator};

/// Values provided to an expression evaluator. A BTreeMap keeps contexts and
/// diagnostics deterministic across platforms.
pub type Environment = BTreeMap<Identifier, f64>;

pub fn evaluate(expression: &Expr, environment: &Environment) -> Result<f64, EvaluationError> {
    let value = match expression {
        Expr::Constant(value) => *value,
        Expr::Symbol(identifier) => *environment
            .get(identifier)
            .ok_or_else(|| EvaluationError::UnknownSymbol(identifier.clone()))?,
        Expr::Unary { operator, operand } => {
            let value = evaluate(operand, environment)?;
            match operator {
                UnaryOperator::Negate => -value,
                UnaryOperator::Exp => value.exp(),
                UnaryOperator::Log if value > 0.0 => value.ln(),
                UnaryOperator::Log => {
                    return Err(EvaluationError::DomainError { operation: "log", input: value });
                }
                UnaryOperator::Sin => value.sin(),
                UnaryOperator::Cos => value.cos(),
            }
        }
        Expr::Binary { operator, left, right } => {
            let left = evaluate(left, environment)?;
            let right = evaluate(right, environment)?;
            match operator {
                BinaryOperator::Add => left + right,
                BinaryOperator::Subtract => left - right,
                BinaryOperator::Multiply => left * right,
                BinaryOperator::Divide if right == 0.0 => {
                    return Err(EvaluationError::DivisionByZero);
                }
                BinaryOperator::Divide => left / right,
                BinaryOperator::Power => left.powf(right),
            }
        }
    };
    if value.is_finite() { Ok(value) } else { Err(EvaluationError::NonFiniteResult) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BinaryOperator, Expr, UnaryOperator};

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn evaluates_a_composed_expression() {
        let expression = Expr::sum(
            Expr::binary(BinaryOperator::Multiply, Expr::constant(2.0), Expr::symbol(id("x"))),
            Expr::unary(UnaryOperator::Negate, Expr::symbol(id("y"))),
        );
        let environment = Environment::from([(id("x"), 3.0), (id("y"), 1.5)]);
        assert_eq!(evaluate(&expression, &environment).unwrap(), 4.5);
    }

    #[test]
    fn refuses_invalid_arithmetic() {
        assert_eq!(
            evaluate(
                &Expr::quotient(Expr::constant(1.0), Expr::constant(0.0)),
                &Environment::new()
            ),
            Err(EvaluationError::DivisionByZero)
        );
    }
}
