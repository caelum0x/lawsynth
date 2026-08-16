use lawsynth_expr::{BinaryOperator, binary_precedence, is_commutative};

#[test]
fn operator_metadata_matches_arithmetic_binding_and_symmetry() {
    assert!(binary_precedence(BinaryOperator::Power) > binary_precedence(BinaryOperator::Multiply));
    assert!(is_commutative(BinaryOperator::Add));
    assert!(!is_commutative(BinaryOperator::Divide));
}
