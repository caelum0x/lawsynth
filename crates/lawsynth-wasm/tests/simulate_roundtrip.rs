use lawsynth_wasm::{Expression, WasmConfig, World, simulate_rk4};
#[test]
fn rk4_tracks_exponential_decay() {
    let world = World::new(
        vec!["x".into()],
        vec![1.0],
        vec![Expression::parse("-x").unwrap()],
    )
    .unwrap();
    let trajectory = simulate_rk4(&world, 0.0, 1.0, 0.01, &WasmConfig::default()).unwrap();
    assert!((trajectory.values.last().unwrap()[0] - (-1.0f64).exp()).abs() < 1e-8);
}
