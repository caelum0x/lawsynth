use lawsynth_opt::{CoordinateConfig, ParameterBounds, coordinate_minimize};

#[test]
fn coordinate_search_clamps_initial_and_proposed_values_to_inclusive_bounds() {
    let bounds = ParameterBounds::new(-1.0, 1.0).unwrap();
    let result = coordinate_minimize(
        &[10.0],
        bounds,
        CoordinateConfig { initial_step: 0.5, minimum_step: 1e-5, max_iterations: 100 },
        |point| (point[0] - 4.0).powi(2),
    )
    .unwrap();
    assert_eq!(result.parameters, vec![1.0]);
    assert_eq!(result.objective, 9.0);
}
