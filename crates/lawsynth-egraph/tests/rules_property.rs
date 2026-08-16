use lawsynth_core::Identifier;
use lawsynth_egraph::normalize;
use lawsynth_expr::Expr;

#[test]
fn normalization_is_idempotent() {
    let x = Expr::symbol(Identifier::new("x").unwrap());
    let once = normalize(Expr::sum(Expr::constant(0.0), x));
    assert_eq!(normalize(once.clone()), once);
}
