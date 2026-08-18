use crate::{BinaryOperator, Expr, UnaryOperator};

/// Renders an expression with explicit parentheses for unambiguous interchange.
pub fn print(expression: &Expr) -> String {
    match expression {
        Expr::Constant(value) => format!("{value:.17e}"),
        Expr::Symbol(id) => id.as_str().to_owned(),
        Expr::Unary { operator, operand } => match operator {
            UnaryOperator::Negate => format!("-({})", print(operand)),
            UnaryOperator::Exp => format!("exp({})", print(operand)),
            UnaryOperator::Log => format!("log({})", print(operand)),
            UnaryOperator::Sin => format!("sin({})", print(operand)),
            UnaryOperator::Cos => format!("cos({})", print(operand)),
        },
        Expr::Binary { operator, left, right } => {
            let operator = match operator {
                BinaryOperator::Add => "+",
                BinaryOperator::Subtract => "-",
                BinaryOperator::Multiply => "*",
                BinaryOperator::Divide => "/",
                BinaryOperator::Power => "^",
            };
            format!("({}{}{})", print(left), operator, print(right))
        }
    }
}
