use lawsynth_plugin_api::SimulationRequest;
use std::hint::black_box;
use std::time::Instant;

fn main() {
    let request = SimulationRequest {
        initial_state: vec![1.0, 2.0],
        times: (0..10_000).map(|i| i as f64 * 0.01).collect(),
    };
    let start = Instant::now();
    for _ in 0..1_000 {
        request.validate().unwrap();
        black_box(());
    }
    println!("1000 request validations in {:?}", start.elapsed());
}
