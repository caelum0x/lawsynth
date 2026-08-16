use lawsynth_differentiate::cubic_spline_derivative;

#[test]
fn spline_derivative_is_deterministic_on_an_irregular_signal() {
    let time: [f64; 5] = [0.0, 0.4, 1.7, 2.0, 4.0];
    let values = time
        .iter()
        .map(|sample| (*sample).sin() + 0.5 * sample)
        .collect::<Vec<_>>();
    let first = cubic_spline_derivative(&time, &values).unwrap();
    let second = cubic_spline_derivative(&time, &values).unwrap();
    assert_eq!(first, second);
    assert!(first.iter().all(|value| value.is_finite()));
}
