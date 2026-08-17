use lawsynth_profile::estimate_delay;

fn main() {
    let source = [0.0, 1.0, 2.0, 3.0, 4.0];
    let delayed = [0.0, 0.0, 1.0, 2.0, 3.0];
    let estimate = estimate_delay(&source, &delayed, 2).unwrap();
    println!("best lag: {} samples (correlation {:.3})", estimate.lag, estimate.correlation);
}
