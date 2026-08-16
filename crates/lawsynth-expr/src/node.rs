use crate::{BinaryOperator, Expr, UnaryOperator};
use lawsynth_core::Identifier;

/// A non-recursive view of an expression root, useful to visitors and UIs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExpressionNode<'a> {
    Constant(f64),
    Symbol(&'a Identifier),
    Unary {
        operator: UnaryOperator,
        operand: &'a Expr,
    },
    Binary {
        operator: BinaryOperator,
        left: &'a Expr,
        right: &'a Expr,
    },
}

impl<'a> From<&'a Expr> for ExpressionNode<'a> {
    fn from(expression: &'a Expr) -> Self {
        match expression {
            Expr::Constant(value) => Self::Constant(*value),
            Expr::Symbol(identifier) => Self::Symbol(identifier),
            Expr::Unary { operator, operand } => Self::Unary {
                operator: *operator,
                operand,
            },
            Expr::Binary {
                operator,
                left,
                right,
            } => Self::Binary {
                operator: *operator,
                left,
                right,
            },
        }
    }
}
