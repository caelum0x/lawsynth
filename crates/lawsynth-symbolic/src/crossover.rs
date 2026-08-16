use lawsynth_expr::Expr;

/// Produces a symmetric crossover child using an additive composition.
/// The IR simplifier removes neutral constants and folds fully constant pairs.
pub fn crossover_sum(left: &Expr, right: &Expr) -> Expr {
    Expr::sum(left.clone(), right.clone()).simplify()
}
