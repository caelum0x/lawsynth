use lawsynth_expr::Expr;

/// Runs the expression IR's safe local simplifier before storage or scoring.
pub fn simplify_candidate(expression: &Expr) -> Expr {
    expression.simplify()
}
