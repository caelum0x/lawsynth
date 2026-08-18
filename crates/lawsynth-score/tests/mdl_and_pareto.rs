//! Black-box coverage for the public MDL objective and the O(n log n)
//! two-objective Pareto front against the general O(n^2) filter.

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_score::{
    CandidateMetrics, DescriptionLength, ModelDescription, description_length, most_parsimonious,
    pareto_front, pareto_front_2d,
};

fn symbol(name: &str) -> Expr {
    Expr::symbol(Identifier::new(name).unwrap())
}

#[test]
fn mdl_selects_the_parsimonious_model_over_an_overfit_alternative() {
    // Simple model: a*x + b (5 nodes, 2 constants), fits with a little residual.
    let simple = ModelDescription::from_expression(&Expr::sum(
        Expr::product(Expr::constant(2.0), symbol("x")),
        Expr::constant(1.0),
    ));
    // Bloated model: many extra operators and constants that fit no better.
    let bloated = ModelDescription::from_expression(&Expr::sum(
        Expr::product(Expr::constant(2.0), symbol("x")),
        Expr::sum(
            Expr::product(Expr::constant(0.0), Expr::product(symbol("x"), symbol("x"))),
            Expr::sum(Expr::constant(1.0), Expr::product(Expr::constant(0.0), symbol("y"))),
        ),
    ));
    let observations = 100;
    let residual = 0.5;

    let simple_dl: DescriptionLength = description_length(observations, residual, &simple).unwrap();
    let bloated_dl: DescriptionLength =
        description_length(observations, residual, &bloated).unwrap();

    // Same fit, so the data terms match and the model term breaks the tie.
    assert_eq!(simple_dl.data_code_length, bloated_dl.data_code_length);
    assert!(simple_dl.total < bloated_dl.total);
    assert_eq!(most_parsimonious(&[bloated_dl, simple_dl]), Some(1));
}

#[test]
fn two_dimensional_front_equals_the_quadratic_front() {
    let candidates = vec![
        CandidateMetrics { mean_squared_error: 0.4, complexity: 2 },
        CandidateMetrics { mean_squared_error: 0.4, complexity: 2 }, // exact duplicate
        CandidateMetrics { mean_squared_error: 0.2, complexity: 5 },
        CandidateMetrics { mean_squared_error: 0.2, complexity: 9 }, // dominated tie on error
        CandidateMetrics { mean_squared_error: 0.1, complexity: 9 },
        CandidateMetrics { mean_squared_error: 0.5, complexity: 5 }, // dominated interior
    ];
    assert_eq!(pareto_front_2d(&candidates), pareto_front(&candidates));
    assert_eq!(pareto_front_2d(&candidates), vec![0, 1, 2, 4]);
}
