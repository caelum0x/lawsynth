//! Weak-form discovery is bit-for-bit deterministic: identical inputs and
//! configuration produce identical output, including on noisy data.

mod common;

use common::{Noise, add_noise, dataset, oscillator};
use lawsynth_weakform::{WeakConfig, weak_discover};

#[test]
fn identical_input_yields_identical_output() {
    let (time, xs, ys) = oscillator(1000, 0.01);
    let data = dataset(time, xs, ys);
    let config = WeakConfig::default();

    let first = weak_discover(&data, &config).unwrap();
    let second = weak_discover(&data, &config).unwrap();

    // Whole-struct equality is exact-bit equality for the f64 fields.
    assert_eq!(first, second);
    assert_eq!(first.coefficients, second.coefficients);
    assert_eq!(first.diagnostics, second.diagnostics);
}

#[test]
fn seeded_noise_is_reproducible_end_to_end() {
    let (time, xs, ys) = oscillator(1000, 0.01);

    let build = || {
        let mut noise = Noise::new(0xDEAD_BEEF_0000_0007);
        let nx = add_noise(&xs, 0.03, &mut noise);
        let ny = add_noise(&ys, 0.03, &mut noise);
        dataset(time.clone(), nx, ny)
    };

    let data_a = build();
    let data_b = build();
    assert_eq!(data_a, data_b, "seeded noise must reproduce the same dataset");

    let config = WeakConfig::default();
    let result_a = weak_discover(&data_a, &config).unwrap();
    let result_b = weak_discover(&data_b, &config).unwrap();
    assert_eq!(result_a, result_b);
}
