use lawsynth_core::Seed;
use lawsynth_sim::{SdeConfig, euler_maruyama};

fn main() {
    let trajectory = euler_maruyama(
        &[1.0],
        SdeConfig { start: 0.0, end: 1.0, step: 0.01, seed: Seed::new(7) },
        |_, state| vec![-state[0]],
        |_, _| vec![0.15],
    )
    .unwrap();
    println!(
        "Euler-Maruyama generated {} samples; x(1)={:.4}",
        trajectory.time.len(),
        trajectory.values.last().unwrap()[0]
    );
}
