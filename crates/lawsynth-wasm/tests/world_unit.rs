use lawsynth_wasm::{Expression, World};
#[test]
fn world_evaluates_a_validated_derivative() {
    let world = World::new(vec!["x".into()], vec![2.0], vec![Expression::parse("-x + t").unwrap()])
        .unwrap();
    assert_eq!(world.derivative_at(3.0, &[2.0]).unwrap(), vec![1.0]);
    assert!(
        World::new(
            vec!["x".into(), "x".into()],
            vec![0.0, 0.0],
            vec![Expression::parse("x").unwrap(), Expression::parse("x").unwrap()]
        )
        .is_err()
    );
}
