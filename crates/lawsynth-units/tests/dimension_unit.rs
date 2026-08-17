use lawsynth_units::Dimension;

#[test]
fn dimensions_compose_and_invert_base_exponents() {
    let velocity = Dimension::LENGTH.divide(Dimension::TIME).unwrap();
    assert_eq!(velocity.exponents(), [1, 0, -1, 0, 0, 0, 0]);
    assert_eq!(velocity.multiply(Dimension::TIME).unwrap(), Dimension::LENGTH);
    assert_eq!(Dimension::LENGTH.pow(2).unwrap().exponents(), [2, 0, 0, 0, 0, 0, 0]);
}
