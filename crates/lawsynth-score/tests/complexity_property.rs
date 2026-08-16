use lawsynth_expr::Expr;
use lawsynth_score::expression_complexity;

#[test]
fn complexity_is_additive_when_expressions_are_composed() {
    let left = Expr::sum(Expr::constant(1.0), Expr::constant(2.0));
    let right = Expr::product(Expr::constant(3.0), Expr::constant(4.0));
    let joined = Expr::sum(left.clone(), right.clone());
    assert_eq!(
        expression_complexity(&joined),
        1 + expression_complexity(&left) + expression_complexity(&right)
    );
}
