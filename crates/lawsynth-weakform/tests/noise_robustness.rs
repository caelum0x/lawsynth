//! The headline demonstration: on noisy data the weak / integral form recovers
//! the true coefficients far better than a naive finite-difference strong-form
//! fit that differentiates the same noisy signal.

mod common;

use common::{Noise, add_noise, central_difference, coefficient_error, dataset, oscillator};
use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_features::FeatureLibrary;
use lawsynth_weakform::{StlsqConfig, WeakConfig, stlsq, weak_discover};

/// Builds the truth coefficient rows aligned to `term_names` for the oscillator
/// `ẋ = y`, `ẏ = -x - 0.3 y`.
fn truth_rows(term_names: &[String]) -> (Vec<f64>, Vec<f64>) {
    let mut x_truth = vec![0.0; term_names.len()];
    let mut y_truth = vec![0.0; term_names.len()];
    for (index, name) in term_names.iter().enumerate() {
        match name.as_str() {
            "x" => y_truth[index] = -1.0,
            "y" => {
                x_truth[index] = 1.0;
                y_truth[index] = -0.3;
            }
            _ => {}
        }
    }
    (x_truth, y_truth)
}

/// A strong-form SINDy fit on noisy data: estimate derivatives by central
/// finite differences, then STLSQ over the identical candidate library. This is
/// the baseline the weak form is meant to beat under noise.
fn strong_form_fit(noisy: &Dataset, config: &WeakConfig) -> (Vec<String>, Vec<Vec<f64>>) {
    let variables = noisy.schema().columns;
    let library =
        FeatureLibrary::polynomial(variables, config.feature_degree, config.include_constant)
            .unwrap();
    let matrix = library.evaluate(noisy).unwrap();
    let term_names: Vec<String> = matrix.terms.iter().map(|term| term.name.clone()).collect();

    let time = noisy.time().values();
    let solve = StlsqConfig {
        threshold: config.threshold,
        ridge: config.ridge,
        max_iterations: config.max_iterations,
    };
    let coefficients = noisy
        .columns()
        .values()
        .map(|column| {
            let derivative = central_difference(time, &column.values);
            stlsq(&matrix.rows, &derivative, &solve).unwrap().coefficients
        })
        .collect();
    (term_names, coefficients)
}

#[test]
fn weak_form_beats_finite_difference_strong_form_under_noise() {
    let (time, xs, ys) = oscillator(1500, 0.01);
    let sigma = 0.02;
    let mut noise = Noise::new(0x5EED_1234_ABCD_0001);
    let noisy_x = add_noise(&xs, sigma, &mut noise);
    let noisy_y = add_noise(&ys, sigma, &mut noise);
    let noisy = dataset(time, noisy_x, noisy_y);

    let config = WeakConfig::default();

    // Weak / integral-form fit.
    let weak = weak_discover(&noisy, &config).unwrap();
    let (weak_x_truth, weak_y_truth) = truth_rows(&weak.term_names);
    let weak_x_error = coefficient_error(&weak.coefficients[0], &weak_x_truth);
    let weak_y_error = coefficient_error(&weak.coefficients[1], &weak_y_truth);
    let weak_error = weak_x_error + weak_y_error;

    // Naive finite-difference strong-form fit on the same noisy data.
    let (strong_names, strong_coefficients) = strong_form_fit(&noisy, &config);
    let (strong_x_truth, strong_y_truth) = truth_rows(&strong_names);
    let strong_x_error = coefficient_error(&strong_coefficients[0], &strong_x_truth);
    let strong_y_error = coefficient_error(&strong_coefficients[1], &strong_y_truth);
    let strong_error = strong_x_error + strong_y_error;

    println!(
        "noise σ={sigma}: weak coeff error = {weak_error:.4} (x {weak_x_error:.4}, y {weak_y_error:.4}); \
         strong coeff error = {strong_error:.4} (x {strong_x_error:.4}, y {strong_y_error:.4})"
    );

    // The weak form must be markedly better and near the truth in absolute terms.
    assert!(
        weak_error < 0.5 * strong_error,
        "weak {weak_error:.4} should be well below strong {strong_error:.4}"
    );
    assert!(weak_error < 0.2, "weak coefficient error {weak_error:.4} should be small");
}

#[test]
fn weak_form_is_stable_across_two_noise_seeds() {
    // The weak error should stay small regardless of the particular noise draw.
    for seed in [0x1111_2222_3333_4444_u64, 0xAAAA_BBBB_CCCC_DDDD_u64] {
        let (time, xs, ys) = oscillator(1200, 0.01);
        let mut noise = Noise::new(seed);
        let noisy_x = add_noise(&xs, 0.02, &mut noise);
        let noisy_y = add_noise(&ys, 0.02, &mut noise);
        let data = Dataset::new(
            TimeAxis::new(time).unwrap(),
            [
                NumericColumn::new(Identifier::new("x").unwrap(), noisy_x),
                NumericColumn::new(Identifier::new("y").unwrap(), noisy_y),
            ],
        )
        .unwrap();
        let weak = weak_discover(&data, &WeakConfig::default()).unwrap();
        let (x_truth, y_truth) = truth_rows(&weak.term_names);
        let error = coefficient_error(&weak.coefficients[0], &x_truth)
            + coefficient_error(&weak.coefficients[1], &y_truth);
        assert!(error < 0.3, "seed {seed:#x}: weak error {error:.4}");
    }
}
