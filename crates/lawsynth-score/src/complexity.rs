use lawsynth_expr::Expr;

/// Counts scalar AST nodes as a deterministic expression-complexity cost.
///
/// Constants and symbols each cost one; every unary or binary operator costs
/// one plus the cost of its operands. This stays stable across printer changes.
pub fn expression_complexity(expression: &Expr) -> usize {
    match expression {
        Expr::Constant(_) | Expr::Symbol(_) => 1,
        Expr::Unary { operand, .. } => 1 + expression_complexity(operand),
        Expr::Binary { left, right, .. } => {
            1 + expression_complexity(left) + expression_complexity(right)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_expression_nodes_not_rendered_text() {
        let expression = Expr::sum(
            Expr::constant(1.0),
            Expr::product(Expr::constant(2.0), Expr::constant(3.0)),
        );
        assert_eq!(expression_complexity(&expression), 5);
    }
}
