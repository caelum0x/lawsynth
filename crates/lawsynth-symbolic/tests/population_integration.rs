use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_symbolic::{Population, ScoredExpression, pareto_by_loss_and_complexity};

#[test]
fn population_canonicalizes_duplicates_before_pareto_ranking() {
    let x = Identifier::new("x").unwrap();
    let x_plus_zero = Expr::sum(Expr::symbol(x.clone()), Expr::constant(0.0));
    let x_plain = Expr::symbol(x);
    let population = Population::new([x_plus_zero, x_plain.clone(), Expr::constant(1.0)]);
    assert_eq!(population.len(), 2);
    let candidates = [
        ScoredExpression { expression: x_plain, loss: 0.1, complexity: 1 },
        ScoredExpression { expression: Expr::constant(1.0), loss: 1.0, complexity: 1 },
    ];
    assert_eq!(pareto_by_loss_and_complexity(&candidates), vec![0]);
}
