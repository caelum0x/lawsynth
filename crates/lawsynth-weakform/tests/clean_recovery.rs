//! Weak-form discovery recovers the exact oscillator coefficients on clean data.

mod common;

use common::{DAMPING, dataset, oscillator};
use lawsynth_weakform::{WeakConfig, weak_discover};

fn term_index(names: &[String], target: &str) -> usize {
    names
        .iter()
        .position(|name| name == target)
        .unwrap_or_else(|| panic!("library is missing term `{target}` in {names:?}"))
}

#[test]
fn recovers_the_damped_oscillator_on_clean_data() {
    let (time, xs, ys) = oscillator(1500, 0.01);
    let data = dataset(time, xs, ys);

    let result = weak_discover(&data, &WeakConfig::default()).unwrap();

    assert_eq!(result.state_names, vec!["x".to_string(), "y".to_string()]);
    let idx_x = term_index(&result.term_names, "x");
    let idx_y = term_index(&result.term_names, "y");

    // ẋ = y  →  coefficient of `y` is 1, coefficient of `x` is 0.
    let x_law = &result.coefficients[0];
    assert!((x_law[idx_y] - 1.0).abs() < 2e-2, "x'→y = {}", x_law[idx_y]);
    assert!(x_law[idx_x].abs() < 2e-2, "x'→x = {}", x_law[idx_x]);

    // ẏ = -x - 0.3 y.
    let y_law = &result.coefficients[1];
    assert!((y_law[idx_x] + 1.0).abs() < 2e-2, "y'→x = {}", y_law[idx_x]);
    assert!((y_law[idx_y] + DAMPING).abs() < 2e-2, "y'→y = {}", y_law[idx_y]);

    // Every term outside the true support was pruned to exactly zero.
    for (index, &coefficient) in x_law.iter().enumerate() {
        if index != idx_y {
            assert!(coefficient.abs() < 2e-2, "spurious x' term {index} = {coefficient}");
        }
    }
    for (index, &coefficient) in y_law.iter().enumerate() {
        if index != idx_x && index != idx_y {
            assert!(coefficient.abs() < 2e-2, "spurious y' term {index} = {coefficient}");
        }
    }

    // Diagnostics are well-formed.
    assert_eq!(result.diagnostics.test_functions, 16);
    assert!(result.diagnostics.condition.is_finite());
    assert!(result.diagnostics.max_residual < 1.0);
}

#[test]
fn renders_a_readable_law() {
    let (time, xs, ys) = oscillator(1500, 0.01);
    let data = dataset(time, xs, ys);
    let result = weak_discover(&data, &WeakConfig::default()).unwrap();
    let rendered = result.laws[0].render();
    assert!(rendered.starts_with("d/dt x = "), "{rendered}");
}
