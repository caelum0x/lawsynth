use lawsynth_core::Identifier;
use lawsynth_symbolic::{Grammar, SymbolicConfig, initialize_population};

#[test]
fn initialized_population_is_unique_deterministic_and_bounded() {
    let grammar = Grammar::scalar([Identifier::new("y").unwrap(), Identifier::new("x").unwrap()]);
    let config = SymbolicConfig { max_depth: 3, max_candidates: 12, include_products: true };
    let first = initialize_population(&grammar, &config);
    let second = initialize_population(&grammar, &config);
    assert_eq!(first, second);
    assert!(first.len() <= config.max_candidates);
    assert!(
        first.expressions().any(|expression| expression.to_canonical_string().contains("symbol:x"))
    );
}
