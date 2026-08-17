//! Recovers known linear operators and their spectra via exact DMD.

use lawsynth_koopman::{Matrix, dmd};

/// Advances a 2-D state by a fixed 2×2 operator.
fn step(a: &[[f64; 2]; 2], x: [f64; 2]) -> [f64; 2] {
    [a[0][0] * x[0] + a[0][1] * x[1], a[1][0] * x[0] + a[1][1] * x[1]]
}

/// Builds aligned `(X, X')` snapshot matrices from a 2-D trajectory.
fn snapshot_pair(states: &[[f64; 2]]) -> (Matrix, Matrix) {
    let m = states.len();
    let rows_x = vec![
        (0..m - 1).map(|k| states[k][0]).collect::<Vec<_>>(),
        (0..m - 1).map(|k| states[k][1]).collect::<Vec<_>>(),
    ];
    let rows_xp = vec![
        (1..m).map(|k| states[k][0]).collect::<Vec<_>>(),
        (1..m).map(|k| states[k][1]).collect::<Vec<_>>(),
    ];
    (Matrix::from_rows(rows_x).unwrap(), Matrix::from_rows(rows_xp).unwrap())
}

fn assert_close(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "expected {b}, got {a} (|Δ|={})", (a - b).abs());
}

#[test]
fn recovers_rotation_decay_operator_to_high_precision() {
    let a = [[0.9, -0.3], [0.3, 0.9]];
    let mut states = vec![[1.0, 0.5]];
    for _ in 0..40 {
        states.push(step(&a, *states.last().unwrap()));
    }
    let (x, x_prime) = snapshot_pair(&states);
    let model = dmd(&x, &x_prime, 2).unwrap();

    let operator = model.operator();
    assert_close(operator.get(0, 0), 0.9, 1e-9);
    assert_close(operator.get(0, 1), -0.3, 1e-9);
    assert_close(operator.get(1, 0), 0.3, 1e-9);
    assert_close(operator.get(1, 1), 0.9, 1e-9);
}

#[test]
fn recovers_complex_eigenvalues_to_1e_10() {
    let a = [[0.9, -0.3], [0.3, 0.9]];
    let mut states = vec![[1.0, 0.5]];
    for _ in 0..40 {
        states.push(step(&a, *states.last().unwrap()));
    }
    let (x, x_prime) = snapshot_pair(&states);
    let model = dmd(&x, &x_prime, 2).unwrap();

    let eigenvalues = model.eigenvalues();
    // The conjugate pair 0.9 ± 0.3 i is recovered (order within the pair is
    // numerically incidental, so match set membership).
    let plus =
        eigenvalues.iter().find(|value| value.im > 0.0).expect("a positive-imaginary eigenvalue");
    let minus =
        eigenvalues.iter().find(|value| value.im < 0.0).expect("a negative-imaginary eigenvalue");
    assert_close(plus.re, 0.9, 1e-10);
    assert_close(plus.im, 0.3, 1e-10);
    assert_close(minus.re, 0.9, 1e-10);
    assert_close(minus.im, -0.3, 1e-10);
}

#[test]
fn predicts_a_matching_trajectory() {
    let a = [[0.9, -0.3], [0.3, 0.9]];
    let mut states = vec![[1.0, 0.5]];
    for _ in 0..40 {
        states.push(step(&a, *states.last().unwrap()));
    }
    let (x, x_prime) = snapshot_pair(&states);
    let model = dmd(&x, &x_prime, 2).unwrap();

    // Roll forward from a fresh initial condition and compare to the truth.
    let start = [2.0, -1.0];
    let horizon = 15;
    let mut truth = Vec::new();
    let mut current = start;
    for _ in 0..horizon {
        current = step(&a, current);
        truth.push(current);
    }
    let predicted = model.predict(&start, horizon).unwrap();
    for (expected, got) in truth.iter().zip(&predicted) {
        assert_close(got[0], expected[0], 1e-8);
        assert_close(got[1], expected[1], 1e-8);
    }
}

#[test]
fn recovers_real_distinct_eigenvalues() {
    // A diagonalisable operator with real eigenvalues (trace 1.45, det 0.505):
    // λ = (1.45 ± √0.0825)/2 ≈ 0.86861407 and 0.58138593.
    let a = [[0.8, 0.15], [0.1, 0.65]];
    let mut states = vec![[1.0, -0.5]];
    for _ in 0..40 {
        states.push(step(&a, *states.last().unwrap()));
    }
    let (x, x_prime) = snapshot_pair(&states);
    let model = dmd(&x, &x_prime, 2).unwrap();
    let operator = model.operator();
    assert_close(operator.get(0, 0), 0.8, 1e-9);
    assert_close(operator.get(0, 1), 0.15, 1e-9);
    assert_close(operator.get(1, 0), 0.1, 1e-9);
    assert_close(operator.get(1, 1), 0.65, 1e-9);
    // Eigenvalues are real; verify via trace and determinant to avoid ordering.
    let eigenvalues = model.eigenvalues();
    assert_close(eigenvalues[0].im, 0.0, 1e-10);
    assert_close(eigenvalues[1].im, 0.0, 1e-10);
    let sum = eigenvalues[0].re + eigenvalues[1].re;
    let product = eigenvalues[0].re * eigenvalues[1].re;
    assert_close(sum, 1.45, 1e-9);
    assert_close(product, 0.505, 1e-9);
}

#[test]
fn continuous_eigenvalues_expose_growth_and_frequency() {
    let a = [[0.9, -0.3], [0.3, 0.9]];
    let mut states = vec![[1.0, 0.5]];
    for _ in 0..40 {
        states.push(step(&a, *states.last().unwrap()));
    }
    let (x, x_prime) = snapshot_pair(&states);
    let model = dmd(&x, &x_prime, 2).unwrap();
    let continuous = model.continuous_eigenvalues(1.0);
    // |λ| = √0.9 < 1 ⇒ decaying ⇒ negative real part; nonzero frequency.
    assert!(continuous[0].re < 0.0, "expected decay, got {}", continuous[0].re);
    assert!(continuous[0].im.abs() > 0.0, "expected oscillation");
}

#[test]
fn rejects_invalid_rank() {
    let a = [[0.9, -0.3], [0.3, 0.9]];
    let mut states = vec![[1.0, 0.5]];
    for _ in 0..10 {
        states.push(step(&a, *states.last().unwrap()));
    }
    let (x, x_prime) = snapshot_pair(&states);
    assert!(dmd(&x, &x_prime, 0).is_err());
    assert!(dmd(&x, &x_prime, 3).is_err());
}
