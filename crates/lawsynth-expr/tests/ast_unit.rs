use lawsynth_expr::{BinaryOperator, Expr, ExpressionConfig};

#[test]
fn structural_limits_count_ast_nodes() {
    let expression = Expr::binary(
        BinaryOperator::Add,
        Expr::constant(1.0),
        Expr::constant(2.0),
    );
    assert!(ExpressionConfig { maximum_nodes: 3 }.accepts(&expression));
    assert!(!ExpressionConfig { maximum_nodes: 2 }.accepts(&expression));
}
