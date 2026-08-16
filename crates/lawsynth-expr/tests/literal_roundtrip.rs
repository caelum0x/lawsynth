use lawsynth_expr::Literal;

#[test]
fn finite_literals_round_trip_and_reject_non_finite_values() {
    let literal = Literal::try_from(-3.25).unwrap();
    assert_eq!(literal.value(), -3.25);
    assert!(Literal::try_from(f64::INFINITY).is_err());
}
