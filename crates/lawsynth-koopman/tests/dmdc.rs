//! Recovers `(A, B)` for a controlled linear system via DMDc.

use lawsynth_koopman::{Matrix, dmdc};

fn assert_close(a: f64, b: f64, tol: f64) {
    assert!((a - b).abs() <= tol, "expected {b}, got {a} (|Δ|={})", (a - b).abs());
}

/// Generates a controlled linear trajectory and returns `(X, X', U)`.
fn controlled_system() -> (Matrix, Matrix, Matrix, [[f64; 2]; 2], [f64; 2]) {
    let a = [[0.8, -0.2], [0.1, 0.95]];
    let b = [0.5, -0.3];
    let horizon = 60;
    // A deterministic, persistently-exciting control signal.
    let control: Vec<f64> =
        (0..horizon).map(|t| (0.3 * t as f64).sin() + 0.5 * (0.11 * t as f64).cos()).collect();

    let mut states = vec![[0.0, 0.0]];
    for (t, &u) in control.iter().enumerate() {
        let x = states[t];
        let next = [
            a[0][0] * x[0] + a[0][1] * x[1] + b[0] * u,
            a[1][0] * x[0] + a[1][1] * x[1] + b[1] * u,
        ];
        states.push(next);
    }

    let m = control.len();
    let rows_x = vec![
        (0..m).map(|k| states[k][0]).collect::<Vec<_>>(),
        (0..m).map(|k| states[k][1]).collect::<Vec<_>>(),
    ];
    let rows_xp = vec![
        (0..m).map(|k| states[k + 1][0]).collect::<Vec<_>>(),
        (0..m).map(|k| states[k + 1][1]).collect::<Vec<_>>(),
    ];
    let rows_u = vec![control.clone()];
    (
        Matrix::from_rows(rows_x).unwrap(),
        Matrix::from_rows(rows_xp).unwrap(),
        Matrix::from_rows(rows_u).unwrap(),
        a,
        b,
    )
}

#[test]
fn recovers_state_and_control_operators() {
    let (x, x_prime, u, a, b) = controlled_system();
    let model = dmdc(&x, &x_prime, &u, 3).unwrap();

    let recovered_a = model.state_operator();
    assert_close(recovered_a.get(0, 0), a[0][0], 1e-8);
    assert_close(recovered_a.get(0, 1), a[0][1], 1e-8);
    assert_close(recovered_a.get(1, 0), a[1][0], 1e-8);
    assert_close(recovered_a.get(1, 1), a[1][1], 1e-8);

    let recovered_b = model.control_operator();
    assert_close(recovered_b.get(0, 0), b[0], 1e-8);
    assert_close(recovered_b.get(1, 0), b[1], 1e-8);
}

#[test]
fn predicts_under_control() {
    let (x, x_prime, u, a, b) = controlled_system();
    let model = dmdc(&x, &x_prime, &u, 3).unwrap();

    let start = [0.3, -0.4];
    let controls: Vec<Vec<f64>> = (0..12).map(|t| vec![(0.2 * t as f64).cos()]).collect();
    let mut truth = Vec::new();
    let mut current = start;
    for control in &controls {
        let u = control[0];
        current = [
            a[0][0] * current[0] + a[0][1] * current[1] + b[0] * u,
            a[1][0] * current[0] + a[1][1] * current[1] + b[1] * u,
        ];
        truth.push(current);
    }
    let predicted = model.predict(&start, &controls).unwrap();
    for (expected, got) in truth.iter().zip(&predicted) {
        assert_close(got[0], expected[0], 1e-7);
        assert_close(got[1], expected[1], 1e-7);
    }
}

#[test]
fn recovers_state_eigenvalues() {
    let (x, x_prime, u, a, _b) = controlled_system();
    let model = dmdc(&x, &x_prime, &u, 3).unwrap();
    let eigenvalues = model.state_eigenvalues().unwrap();
    // Trace 1.75, det 0.78 ⇒ eigenvalues 0.875 ± √(0.765625 − 0.78) i.
    let trace: f64 = a[0][0] + a[1][1];
    let det = a[0][0] * a[1][1] - a[0][1] * a[1][0];
    let sum: f64 = eigenvalues.iter().map(|value| value.re).sum();
    let product = eigenvalues[0].mul(eigenvalues[1]);
    assert_close(sum, trace, 1e-7);
    assert_close(product.re, det, 1e-7);
    assert_close(product.im, 0.0, 1e-7);
}
