use crate::ExpressionAnalysis;
use lawsynth_expr::Expr;

/// Stable extraction cost: scalar AST node count.
pub fn expression_cost(expression: &Expr) -> usize {
    ExpressionAnalysis::inspect(expression).nodes
}
