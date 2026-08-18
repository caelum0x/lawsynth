use lawsynth_egraph::ExpressionLanguage;
use lawsynth_expr::Expr;

#[test]
fn language_classifies_expression_roots() {
    assert_eq!(ExpressionLanguage::from(&Expr::constant(1.0)), ExpressionLanguage::Constant);
}
