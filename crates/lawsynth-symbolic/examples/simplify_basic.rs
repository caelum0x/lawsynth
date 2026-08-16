use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_symbolic::simplify_candidate;
fn main() {
    let x = Expr::symbol(Identifier::new("x").unwrap());
    let simplified = simplify_candidate(&Expr::sum(x, Expr::constant(0.0)));
    println!("canonical candidate: {}", simplified.to_canonical_string());
}
