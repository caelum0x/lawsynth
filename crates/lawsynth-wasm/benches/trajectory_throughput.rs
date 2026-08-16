use lawsynth_wasm::{Expression, WasmConfig, World, simulate_rk4};
use std::hint::black_box;
use std::time::Instant;
fn main() {
    let world = World::new(
        vec!["x".into()],
        vec![1.0],
        vec![Expression::parse("-x").unwrap()],
    )
    .unwrap();
    let start = Instant::now();
    let mut samples = 0;
    for _ in 0..100 {
        let trajectory = simulate_rk4(&world, 0.0, 10.0, 0.01, &WasmConfig::default()).unwrap();
        samples += trajectory.len();
        black_box(trajectory);
    }
    println!(
        "RK4 trajectories: {:.0} samples/s",
        samples as f64 / start.elapsed().as_secs_f64()
    );
}
