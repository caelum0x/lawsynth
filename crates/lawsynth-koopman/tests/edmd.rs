//! EDMD lifts a mildly nonlinear system and predicts better than raw DMD.

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_koopman::{PolynomialDictionary, dmd, edmd, snapshots_from_dataset};

/// A mildly nonlinear scalar map with a quadratic term.
fn nonlinear(x: f64) -> f64 {
    0.9 * x + 0.05 * x * x
}

fn build_dataset(initial: f64, steps: usize) -> Dataset {
    let mut values = vec![initial];
    for _ in 0..steps {
        values.push(nonlinear(*values.last().unwrap()));
    }
    let time = TimeAxis::new((0..values.len()).map(|i| i as f64).collect()).unwrap();
    let column = NumericColumn::new(Identifier::new("x").unwrap(), values);
    Dataset::new(time, [column]).unwrap()
}

#[test]
fn edmd_predicts_the_nonlinear_map_far_better_than_dmd() {
    let dataset = build_dataset(0.5, 40);

    // EDMD with a degree-2 polynomial dictionary [1, x, x²].
    let dictionary = PolynomialDictionary::new(1, 2).unwrap();
    let edmd_model = edmd(&dataset, &dictionary, 3).unwrap();

    // Raw DMD on the bare scalar state (a single linear coefficient).
    let (x, x_prime) = snapshots_from_dataset(&dataset).unwrap();
    let dmd_model = dmd(&x, &x_prime, 1).unwrap();

    let start = 0.5;
    let horizon = 20;
    let mut truth = Vec::new();
    let mut current = start;
    for _ in 0..horizon {
        current = nonlinear(current);
        truth.push(current);
    }

    let edmd_prediction = edmd_model.predict(&[start], horizon).unwrap();
    let dmd_prediction = dmd_model.predict(&[start], horizon).unwrap();

    let edmd_error: f64 = truth.iter().zip(&edmd_prediction).map(|(t, p)| (t - p[0]).abs()).sum();
    let dmd_error: f64 = truth.iter().zip(&dmd_prediction).map(|(t, p)| (t - p[0]).abs()).sum();

    assert!(edmd_error < 1e-6, "EDMD error too large: {edmd_error}");
    assert!(edmd_error < dmd_error * 1e-2, "EDMD ({edmd_error}) should crush DMD ({dmd_error})");
}

#[test]
fn dictionary_orders_the_constant_first() {
    let dictionary = PolynomialDictionary::new(2, 2).unwrap();
    // Two variables, total degree ≤ 2 ⇒ features: 1, x, y, x², xy, y² = 6.
    assert_eq!(dictionary.feature_count(), 6);
    let lifted = dictionary.lift(&[2.0, 3.0]).unwrap();
    assert_eq!(lifted[0], 1.0); // constant term first
    assert!(lifted.contains(&4.0)); // x²
    assert!(lifted.contains(&9.0)); // y²
    assert!(lifted.contains(&6.0)); // xy
}

#[test]
fn edmd_eigenvalue_captures_the_linear_growth() {
    let dataset = build_dataset(0.4, 30);
    let dictionary = PolynomialDictionary::new(1, 2).unwrap();
    let model = edmd(&dataset, &dictionary, 3).unwrap();
    // The constant observable is fixed ⇒ an eigenvalue of exactly 1 is present.
    let has_unit_eigenvalue = model
        .eigenvalues()
        .iter()
        .any(|value| (value.re - 1.0).abs() < 1e-6 && value.im.abs() < 1e-6);
    assert!(has_unit_eigenvalue, "constant observable should give a unit eigenvalue");
}

#[test]
fn rejects_dictionary_state_mismatch() {
    let dataset = build_dataset(0.5, 10);
    // Dataset has one column but dictionary expects two variables.
    let dictionary = PolynomialDictionary::new(2, 2).unwrap();
    assert!(edmd(&dataset, &dictionary, 2).is_err());
}
