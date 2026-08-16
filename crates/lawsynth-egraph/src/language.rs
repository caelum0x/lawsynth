use lawsynth_expr::{BinaryOperator, Expr, UnaryOperator};

/// A compact operator classification for inspecting expression-language usage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExpressionLanguage {
    Constant,
    Symbol,
    Unary(UnaryOperator),
    Binary(BinaryOperator),
}

impl From<&Expr> for ExpressionLanguage {
    fn from(expression: &Expr) -> Self {
        match expression {
            Expr::Constant(_) => Self::Constant,
            Expr::Symbol(_) => Self::Symbol,
            Expr::Unary { operator, .. } => Self::Unary(*operator),
            Expr::Binary { operator, .. } => Self::Binary(*operator),
        }
    }
}
