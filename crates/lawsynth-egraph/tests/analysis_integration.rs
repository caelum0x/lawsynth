use lawsynth_core::Identifier;
use lawsynth_egraph::ExpressionAnalysis;
use lawsynth_expr::Expr;

#[test]
fn analysis_collects_nodes_and_symbols() {
    let x = Identifier::new("x").unwrap();
    let analysis = ExpressionAnalysis::inspect(&Expr::sum(Expr::symbol(x.clone()), Expr::constant(1.0)));
    assert_eq!(analysis.nodes, 3);
    assert!(analysis.symbols.contains(&x));
}
