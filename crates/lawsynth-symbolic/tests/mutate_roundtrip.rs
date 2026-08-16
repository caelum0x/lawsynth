use lawsynth_core::Identifier;
use lawsynth_expr::{Environment, Expr, evaluate};
use lawsynth_symbolic::{crossover_sum, replace_symbol, simplify_candidate};

#[test]
fn mutation_replaces_every_terminal_without_changing_the_expression_shape() {
    let x = Identifier::new("x").unwrap();
    let y = Identifier::new("y").unwrap();
    let expression = Expr::product(
        Expr::symbol(x.clone()),
        Expr::sum(Expr::symbol(x.clone()), Expr::constant(1.0)),
    );
    let mutated = replace_symbol(&expression, &x, y.clone());
    assert_eq!(
        evaluate(&mutated, &Environment::from([(y.clone(), 3.0)])).unwrap(),
        12.0
    );
    let child = simplify_candidate(&crossover_sum(&mutated, &Expr::constant(0.0)));
    assert_eq!(child, mutated);
}
