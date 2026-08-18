//! Integration tests for deterministic balanced-truncation model reduction.

use lawsynth_modelreduce::{
    Matrix, ModelReduceError, ReductionSpec, balanced_truncation, controllability_gramian, eigen,
    hankel_singular_values, observability_gramian,
};

// ---------------------------------------------------------------------------
// Small helpers (integration tests only see the public API).
// ---------------------------------------------------------------------------

fn matrix(rows: &[&[f64]]) -> Matrix {
    Matrix::from_rows(rows.iter().map(|row| row.to_vec()).collect()).unwrap()
}

fn max_abs_diff(a: &Matrix, b: &Matrix) -> f64 {
    let mut best = 0.0_f64;
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            best = best.max((a.get(i, j) - b.get(i, j)).abs());
        }
    }
    best
}

/// Residual `A W + W Aᵀ + Q` in the max norm (should vanish for a valid gramian).
fn lyapunov_residual(a: &Matrix, w: &Matrix, q: &Matrix) -> f64 {
    let aw = a.matmul(w).unwrap();
    let wat = w.matmul(&a.transpose()).unwrap();
    let mut best = 0.0_f64;
    for i in 0..a.rows() {
        for j in 0..a.cols() {
            let value = aw.get(i, j) + wat.get(i, j) + q.get(i, j);
            best = best.max(value.abs());
        }
    }
    best
}

/// Simulates `ẋ = A x + B u`, `y = C x` from rest under a unit step `u ≡ 1`
/// (single input, single output) with fixed-step RK4, returning `y` samples.
fn step_response(a: &Matrix, b: &Matrix, c: &Matrix, dt: f64, steps: usize) -> Vec<f64> {
    let n = a.rows();
    let deriv = |x: &[f64]| -> Vec<f64> {
        let ax = a.mat_vec(x).unwrap();
        (0..n).map(|i| ax[i] + b.get(i, 0)).collect()
    };
    let mut x = vec![0.0; n];
    let mut output = Vec::with_capacity(steps + 1);
    let observe = |x: &[f64]| -> f64 { (0..n).map(|i| c.get(0, i) * x[i]).sum() };
    output.push(observe(&x));
    for _ in 0..steps {
        let k1 = deriv(&x);
        let x2: Vec<f64> = (0..n).map(|i| x[i] + 0.5 * dt * k1[i]).collect();
        let k2 = deriv(&x2);
        let x3: Vec<f64> = (0..n).map(|i| x[i] + 0.5 * dt * k2[i]).collect();
        let k3 = deriv(&x3);
        let x4: Vec<f64> = (0..n).map(|i| x[i] + dt * k3[i]).collect();
        let k4 = deriv(&x4);
        for i in 0..n {
            x[i] += dt / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
        }
        output.push(observe(&x));
    }
    output
}

fn max_response_error(reference: &[f64], candidate: &[f64]) -> f64 {
    reference.iter().zip(candidate).map(|(a, b)| (a - b).abs()).fold(0.0_f64, f64::max)
}

/// A 3-state system whose third mode is only weakly controllable/observable, so
/// it carries a tiny Hankel singular value and should be truncated first.
fn weak_mode_system() -> (Matrix, Matrix, Matrix) {
    let a = matrix(&[&[-1.0, 0.0, 0.0], &[0.0, -2.0, 0.0], &[0.0, 0.0, -3.0]]);
    let b = matrix(&[&[1.0], &[1.0], &[0.001]]);
    let c = matrix(&[&[1.0, 1.0, 0.001]]);
    (a, b, c)
}

// ---------------------------------------------------------------------------
// Hankel singular values against hand/analytic values.
// ---------------------------------------------------------------------------

#[test]
fn scalar_hankel_singular_value_is_one_half() {
    // ẋ = −x, B = 1, C = 1 ⇒ Wc = Wo = 1/2 ⇒ single HSV = 1/2.
    let a = matrix(&[&[-1.0]]);
    let b = matrix(&[&[1.0]]);
    let c = matrix(&[&[1.0]]);
    let sigma = hankel_singular_values(&a, &b, &c).unwrap();
    assert_eq!(sigma.len(), 1);
    assert!((sigma[0] - 0.5).abs() < 1e-12, "sigma = {:?}", sigma);
}

#[test]
fn second_order_hankel_singular_values_match_analytic() {
    // A = diag(-1,-2), B = C = [1,1]. Then Wc = Wo = W = [[1/2,1/3],[1/3,1/4]],
    // and the HSVs equal the eigenvalues of W: 3/8 ± sqrt(1/64 + 1/9).
    let a = matrix(&[&[-1.0, 0.0], &[0.0, -2.0]]);
    let b = matrix(&[&[1.0], &[1.0]]);
    let c = matrix(&[&[1.0, 1.0]]);
    let sigma = hankel_singular_values(&a, &b, &c).unwrap();

    let spread = (1.0_f64 / 64.0 + 1.0 / 9.0).sqrt();
    let expected_hi = 0.375 + spread;
    let expected_lo = 0.375 - spread;
    assert!((sigma[0] - expected_hi).abs() < 1e-9, "sigma = {:?}", sigma);
    assert!((sigma[1] - expected_lo).abs() < 1e-9, "sigma = {:?}", sigma);
    // Non-increasing order.
    assert!(sigma[0] > sigma[1]);
}

// ---------------------------------------------------------------------------
// Gramian correctness: the solved Wc, Wo satisfy their Lyapunov equations.
// ---------------------------------------------------------------------------

#[test]
fn gramians_satisfy_their_lyapunov_equations() {
    let (a, b, c) = weak_mode_system();
    let wc = controllability_gramian(&a, &b).unwrap();
    let wo = observability_gramian(&a, &c).unwrap();

    let bbt = b.matmul(&b.transpose()).unwrap();
    let ctc = c.transpose().matmul(&c).unwrap();
    // A Wc + Wc Aᵀ + B Bᵀ = 0.
    assert!(lyapunov_residual(&a, &wc, &bbt) < 1e-12);
    // Aᵀ Wo + Wo A + Cᵀ C = 0 (use Aᵀ as the "A" of the residual form).
    assert!(lyapunov_residual(&a.transpose(), &wo, &ctc) < 1e-12);
}

// ---------------------------------------------------------------------------
// Balanced realization (k = n): both gramians are diagonal and equal diag(σ).
// ---------------------------------------------------------------------------

#[test]
fn full_balanced_realization_has_diagonal_equal_gramians() {
    let a = matrix(&[&[-1.0, 0.0], &[0.0, -2.0]]);
    let b = matrix(&[&[1.0], &[1.0]]);
    let c = matrix(&[&[1.0, 1.0]]);

    let balanced = balanced_truncation(&a, &b, &c, &ReductionSpec::Order(2)).unwrap();
    let wc = controllability_gramian(&balanced.a, &balanced.b).unwrap();
    let wo = observability_gramian(&balanced.a, &balanced.c).unwrap();

    let mut sigma_diag = Matrix::zeros(2, 2);
    for i in 0..2 {
        sigma_diag.set(i, i, balanced.hankel_singular_values[i]);
    }
    // Both transformed gramians equal diag(σ).
    assert!(max_abs_diff(&wc, &sigma_diag) < 1e-8, "Wc balanced = {:?}", wc);
    assert!(max_abs_diff(&wo, &sigma_diag) < 1e-8, "Wo balanced = {:?}", wo);
}

// ---------------------------------------------------------------------------
// Truncating a weak mode preserves the response; more states ⇒ smaller error.
// ---------------------------------------------------------------------------

#[test]
fn weak_mode_truncation_preserves_response_monotonically() {
    let (a, b, c) = weak_mode_system();
    let full = step_response(&a, &b, &c, 0.01, 1500);

    let reduced2 = balanced_truncation(&a, &b, &c, &ReductionSpec::Order(2)).unwrap();
    let reduced1 = balanced_truncation(&a, &b, &c, &ReductionSpec::Order(1)).unwrap();

    let response2 = step_response(&reduced2.a, &reduced2.b, &reduced2.c, 0.01, 1500);
    let response1 = step_response(&reduced1.a, &reduced1.b, &reduced1.c, 0.01, 1500);

    let error2 = max_response_error(&full, &response2);
    let error1 = max_response_error(&full, &response1);

    // Keeping 2 states tracks the full response tightly...
    assert!(error2 < 1e-2, "k=2 error {error2} too large");
    // ...and strictly better than keeping only 1 (monotone in retained order).
    assert!(error2 < error1, "expected error2 {error2} < error1 {error1}");
    // Dropping the dominant second mode is visibly worse.
    assert!(error1 > 1e-2, "k=1 error {error1} unexpectedly small");
}

// ---------------------------------------------------------------------------
// Stability is preserved: the reduced A is still Hurwitz.
// ---------------------------------------------------------------------------

#[test]
fn reduced_model_is_hurwitz() {
    let (a, b, c) = weak_mode_system();
    let reduced = balanced_truncation(&a, &b, &c, &ReductionSpec::Order(2)).unwrap();
    let spectrum = eigen(&reduced.a).unwrap();
    for value in &spectrum.values {
        assert!(value.re < 0.0, "reduced eigenvalue not stable: {value}");
    }
}

// ---------------------------------------------------------------------------
// The a priori error bound equals 2·Σ of the truncated Hankel singular values.
// ---------------------------------------------------------------------------

#[test]
fn error_bound_is_twice_the_truncated_tail() {
    let (a, b, c) = weak_mode_system();
    let reduced = balanced_truncation(&a, &b, &c, &ReductionSpec::Order(2)).unwrap();
    let tail: f64 = reduced.hankel_singular_values[2..].iter().sum();
    assert!((reduced.error_bound() - 2.0 * tail).abs() < 1e-15);
    // With one weak mode dropped, the bound is small but positive.
    assert!(reduced.error_bound() > 0.0);
    assert!(reduced.error_bound() < 1e-2);
}

// ---------------------------------------------------------------------------
// Energy-tolerance order selection.
// ---------------------------------------------------------------------------

#[test]
fn energy_tolerance_drops_the_weak_mode() {
    let (a, b, c) = weak_mode_system();
    // A 1% tail-energy budget is enough to shed the tiny third HSV, not the rest.
    let reduced = balanced_truncation(&a, &b, &c, &ReductionSpec::EnergyTolerance(0.01)).unwrap();
    assert_eq!(reduced.order, 2);
    assert_eq!(reduced.a.rows(), 2);
    // The full Hankel spectrum is still reported (length n = 3).
    assert_eq!(reduced.hankel_singular_values.len(), 3);
}

// ---------------------------------------------------------------------------
// Determinism: identical inputs ⇒ bit-identical reduced model and σ.
// ---------------------------------------------------------------------------

#[test]
fn reduction_is_bit_identical_across_runs() {
    let (a, b, c) = weak_mode_system();
    let first = balanced_truncation(&a, &b, &c, &ReductionSpec::Order(2)).unwrap();
    let second = balanced_truncation(&a, &b, &c, &ReductionSpec::Order(2)).unwrap();

    let bits = |m: &Matrix| -> Vec<u64> {
        let mut out = Vec::new();
        for i in 0..m.rows() {
            for j in 0..m.cols() {
                out.push(m.get(i, j).to_bits());
            }
        }
        out
    };
    assert_eq!(bits(&first.a), bits(&second.a));
    assert_eq!(bits(&first.b), bits(&second.b));
    assert_eq!(bits(&first.c), bits(&second.c));
    let sigma_bits = |m: &[f64]| m.iter().map(|v| v.to_bits()).collect::<Vec<_>>();
    assert_eq!(
        sigma_bits(&first.hankel_singular_values),
        sigma_bits(&second.hankel_singular_values)
    );
    assert_eq!(first.order, second.order);
}

// ---------------------------------------------------------------------------
// Precondition and shape errors.
// ---------------------------------------------------------------------------

#[test]
fn unstable_system_is_rejected() {
    // A has an eigenvalue at +1 ⇒ not Hurwitz.
    let a = matrix(&[&[1.0, 0.0], &[0.0, -1.0]]);
    let b = matrix(&[&[1.0], &[1.0]]);
    let c = matrix(&[&[1.0, 1.0]]);
    assert_eq!(
        balanced_truncation(&a, &b, &c, &ReductionSpec::Order(1)),
        Err(ModelReduceError::NotStable)
    );
    assert_eq!(hankel_singular_values(&a, &b, &c), Err(ModelReduceError::NotStable));
}

#[test]
fn dimension_mismatches_are_reported() {
    let a = matrix(&[&[-1.0, 0.0], &[0.0, -2.0]]);
    // B with the wrong number of rows.
    let bad_b = matrix(&[&[1.0]]);
    let good_c = matrix(&[&[1.0, 1.0]]);
    assert_eq!(
        balanced_truncation(&a, &bad_b, &good_c, &ReductionSpec::Order(1)),
        Err(ModelReduceError::InputDimensionMismatch)
    );

    // C with the wrong number of columns.
    let good_b = matrix(&[&[1.0], &[1.0]]);
    let bad_c = matrix(&[&[1.0]]);
    assert_eq!(
        balanced_truncation(&a, &good_b, &bad_c, &ReductionSpec::Order(1)),
        Err(ModelReduceError::OutputDimensionMismatch)
    );

    // Non-square A.
    let non_square = matrix(&[&[-1.0, 0.0, 0.0], &[0.0, -2.0, 0.0]]);
    assert_eq!(
        balanced_truncation(&non_square, &good_b, &good_c, &ReductionSpec::Order(1)),
        Err(ModelReduceError::NonSquareState)
    );
}

#[test]
fn invalid_order_is_rejected() {
    let (a, b, c) = weak_mode_system();
    assert_eq!(
        balanced_truncation(&a, &b, &c, &ReductionSpec::Order(0)),
        Err(ModelReduceError::InvalidOrder)
    );
    assert_eq!(
        balanced_truncation(&a, &b, &c, &ReductionSpec::Order(4)),
        Err(ModelReduceError::InvalidOrder)
    );
}
