use lawsynth_core::Identifier;
use lawsynth_expr::{Expr, ExpressionNode};

#[test]
fn node_view_exposes_symbol_without_cloning_its_expression() {
    let identifier = Identifier::new("alpha").unwrap();
    let expression = Expr::symbol(identifier);
    assert!(matches!(ExpressionNode::from(&expression), ExpressionNode::Symbol(id) if id.as_str() == "alpha"));
}
