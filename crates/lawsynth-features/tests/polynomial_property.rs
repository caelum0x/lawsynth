use lawsynth_core::Identifier;
use lawsynth_features::FeatureLibrary;

#[test]
fn polynomial_term_counts_follow_the_multiset_combinatorics() {
    let variables = [Identifier::new("x").unwrap(), Identifier::new("y").unwrap()];
    for (degree, expected) in [(0, 1), (1, 3), (2, 6), (3, 10)] {
        let library = FeatureLibrary::polynomial(variables.clone(), degree, true).unwrap();
        assert_eq!(library.terms().len(), expected);
    }
    assert_eq!(FeatureLibrary::polynomial(variables, 3, false).unwrap().terms().len(), 9);
}
