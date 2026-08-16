use lawsynth_core::Identifier;
use lawsynth_expr::Expr;

/// Replaces every matching terminal in an expression, preserving its shape.
pub fn replace_symbol(expression: &Expr, from: &Identifier, to: Identifier) -> Expr {
    match expression {
        Expr::Constant(value) => Expr::constant(*value),
        Expr::Symbol(symbol) => Expr::symbol(if symbol == from { to } else { symbol.clone() }),
        Expr::Unary { operator, operand } => {
            Expr::unary(*operator, replace_symbol(operand, from, to))
        }
        Expr::Binary {
            operator,
            left,
            right,
        } => Expr::binary(
            *operator,
            replace_symbol(left, from, to.clone()),
            replace_symbol(right, from, to),
        ),
    }
}
