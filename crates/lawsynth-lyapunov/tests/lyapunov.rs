//! Integration tests for the Benettin/QR Lyapunov-spectrum estimator.
//!
//! The suite pins correctness against cases with a known spectrum or a known
//! divergence, using tolerances appropriate to a **time-averaged** estimator
//! rather than machine precision:
//!
//! - **Linear decay** (`ẋ = −x`; `ẋ = −x, ẏ = −2y`): the analytic exponents.
//! - **Harmonic oscillator** (`ẋ = y, ẏ = −x`): conservative, both exponents ≈ 0
//!   — a discriminating "no separation, no chaos" check.
//! - **Damped oscillator** (`ẋ = y, ẏ = −x − 0.3y`): both exponents negative and
//!   the exponent **sum** equals the constant trace `−0.3` (sum-equals-divergence).
//! - **Lorenz** (`σ=10, ρ=28, β=8/3`): a positive largest exponent (chaos), a
//!   middle exponent ≈ 0, and a **sum** matching the constant divergence
//!   `−(σ+1+β) ≈ −13.667` tightly — the reliable check — while the individual
//!   chaotic exponent gets a broad, honest tolerance.
//!
//! Determinism (bit-identical output) and the typed error paths are exercised too.
//!
//! Integration lengths are stated per test. The Lorenz run integrates 70 000
//! fixed steps of `dt = 0.01` (to `t = 700`, averaging over ≈ 600 time units)
//! with reorthonormalization every 10 steps.

mod common;

use common::{
    Model, damped_oscillator, harmonic_oscillator, id, linear_decay_1d, linear_decay_2d, lorenz,
    lorenz_divergence,
};
use lawsynth_expr::Expr;
use lawsynth_lyapunov::{LyapunovConfig, LyapunovError, LyapunovReport, lyapunov_spectrum};

fn run(model: &Model, initial: &[f64], config: &LyapunovConfig) -> LyapunovReport {
    lyapunov_spectrum(&model.fields, &model.states, initial, config).unwrap()
}

// --- Linear decay: exact analytic spectra ------------------------------------

#[test]
fn linear_decay_1d_exponent_is_minus_one() {
    // ẋ = -x, exact exponent -1. Short run (t = 40) suffices for a linear field.
    let config = LyapunovConfig::default().with_steps(4000);
    let report = run(&linear_decay_1d(), &[1.0], &config);
    assert_eq!(report.dimension(), 1);
    assert!((report.largest() - (-1.0)).abs() < 1e-3, "got {}", report.largest());
}

#[test]
fn linear_decay_2d_spectrum_is_minus_one_and_minus_two() {
    // ẋ = -x, ẏ = -2y: exact spectrum {-1, -2}, sum -3.
    let config = LyapunovConfig::default().with_steps(4000);
    let report = run(&linear_decay_2d(), &[1.0, 1.0], &config);
    let exponents = report.exponents();
    assert_eq!(exponents.len(), 2);
    assert!((exponents[0] - (-1.0)).abs() < 1e-3, "λ1 = {}", exponents[0]);
    assert!((exponents[1] - (-2.0)).abs() < 1e-3, "λ2 = {}", exponents[1]);
    assert!((report.sum() - (-3.0)).abs() < 1e-3, "sum = {}", report.sum());
}

// --- Harmonic oscillator: conservative, both exponents zero ------------------

#[test]
fn harmonic_oscillator_both_exponents_are_zero() {
    // ẋ = y, ẏ = -x: energy-conserving rotation, no separation. Both ≈ 0.
    // A small step keeps the (tiny) RK4 amplitude drift far below tolerance.
    let config = LyapunovConfig::new(0.005, 40_000, 10, 0.1); // t = 200
    let report = run(&harmonic_oscillator(), &[1.0, 0.0], &config);
    for &lambda in report.exponents() {
        assert!(lambda.abs() < 1e-3, "exponent {lambda} is not ≈ 0");
    }
}

#[test]
fn harmonic_oscillator_reports_no_chaos() {
    // The discriminating check: the largest exponent must NOT be positive beyond
    // tolerance, so a conservative system is correctly read as non-chaotic.
    let config = LyapunovConfig::new(0.005, 40_000, 10, 0.1);
    let report = run(&harmonic_oscillator(), &[1.0, 0.0], &config);
    assert!(report.largest() < 1e-3, "largest {} looks spuriously positive", report.largest());
}

// --- Damped oscillator: sum equals the constant trace ------------------------

#[test]
fn damped_oscillator_sum_matches_trace() {
    // ẋ = y, ẏ = -x - 0.3y: tr J = -0.3 everywhere, so Σλ_i = -0.3 tightly.
    let config = LyapunovConfig::new(0.01, 20_000, 10, 0.1); // t = 200
    let report = run(&damped_oscillator(), &[1.0, 0.0], &config);
    assert!((report.sum() - (-0.3)).abs() < 1e-3, "sum = {}", report.sum());
}

#[test]
fn damped_oscillator_both_exponents_negative() {
    // Both real parts are -0.15 (a decaying spiral); both exponents are negative.
    let config = LyapunovConfig::new(0.01, 20_000, 10, 0.1);
    let report = run(&damped_oscillator(), &[1.0, 0.0], &config);
    for &lambda in report.exponents() {
        assert!(lambda < 0.0, "exponent {lambda} should be negative");
    }
    // Each converges toward -0.15 with a modest tolerance.
    for &lambda in report.exponents() {
        assert!((lambda - (-0.15)).abs() < 2e-2, "exponent {lambda} not ≈ -0.15");
    }
}

// --- Lorenz: chaos, and the tight sum-equals-divergence identity -------------

/// A single Lorenz run with the reference configuration, shared by the Lorenz
/// tests. `t = 700`, averaging over ≈ 600 time units, reorth every 10 steps.
fn lorenz_report() -> LyapunovReport {
    let config = LyapunovConfig::new(0.01, 70_000, 10, 0.15);
    run(&lorenz(), &[1.0, 1.0, 1.0], &config)
}

#[test]
fn lorenz_spectrum_signature() {
    let report = lorenz_report();
    let exponents = report.exponents();
    assert_eq!(exponents.len(), 3);

    // Largest exponent is positive (chaos), near the textbook 0.906 — broad
    // tolerance, honestly reflecting the slow convergence of a chaotic exponent.
    assert!(exponents[0] > 0.4, "largest {} is not clearly positive", exponents[0]);
    assert!((exponents[0] - 0.906).abs() < 0.25, "largest {} far from 0.906", exponents[0]);

    // Middle exponent (the flow direction) is ≈ 0.
    assert!(exponents[1].abs() < 0.05, "middle {} is not ≈ 0", exponents[1]);

    // The reliable, tight check: Σλ_i equals the constant divergence -(σ+1+β).
    assert!(
        (report.sum() - lorenz_divergence()).abs() < 0.05,
        "sum {} != divergence {}",
        report.sum(),
        lorenz_divergence()
    );
}

#[test]
fn lorenz_is_chaotic_with_fractional_dimension() {
    let report = lorenz_report();
    // Chaos: a strictly positive largest exponent.
    assert!(report.largest() > 0.0, "Lorenz should be chaotic, got {}", report.largest());
    // The Kaplan–Yorke dimension of the Lorenz attractor is ≈ 2.06 — strictly
    // between 2 and 3 (a strange attractor), not an integer.
    let dimension = report.kaplan_yorke_dimension();
    assert!(dimension > 2.0 && dimension < 3.0, "D_KY {dimension} not in (2, 3)");
}

// --- Determinism -------------------------------------------------------------

#[test]
fn identical_inputs_yield_bit_identical_reports() {
    // A short Lorenz run (bit-identity does not require convergence).
    let config = LyapunovConfig::new(0.01, 3000, 10, 0.1);
    let first = run(&lorenz(), &[1.0, 1.0, 1.0], &config);
    let second = run(&lorenz(), &[1.0, 1.0, 1.0], &config);
    assert_eq!(first.to_canonical_string(), second.to_canonical_string());
}

#[test]
fn identical_inputs_yield_bit_identical_exponents() {
    let config = LyapunovConfig::new(0.01, 3000, 10, 0.1);
    let first = run(&lorenz(), &[1.0, 1.0, 1.0], &config);
    let second = run(&lorenz(), &[1.0, 1.0, 1.0], &config);
    for (a, b) in first.exponents().iter().zip(second.exponents()) {
        assert_eq!(a.to_bits(), b.to_bits(), "exponents differ at the bit level");
    }
    assert_eq!(first.sum().to_bits(), second.sum().to_bits());
    assert_eq!(first.kaplan_yorke_dimension().to_bits(), second.kaplan_yorke_dimension().to_bits());
}

// --- Error paths -------------------------------------------------------------

#[test]
fn rejects_dimension_mismatch() {
    let model = harmonic_oscillator();
    let config = LyapunovConfig::default();
    let error = lyapunov_spectrum(&model.fields, &model.states, &[1.0], &config).unwrap_err();
    assert_eq!(error, LyapunovError::DimensionMismatch { states: 2, initial: 1 });
}

#[test]
fn rejects_non_autonomous_field() {
    // ẋ = a·x references the free symbol `a`, which is not a state.
    let x = id("x");
    let a = id("a");
    let field = Expr::product(Expr::symbol(a.clone()), Expr::symbol(x.clone()));
    let config = LyapunovConfig::default();
    let error = lyapunov_spectrum(&[(x.clone(), field)], &[x], &[1.0], &config).unwrap_err();
    assert_eq!(error, LyapunovError::UnknownSymbol(a));
}

#[test]
fn rejects_non_positive_dt() {
    let model = linear_decay_1d();
    let config = LyapunovConfig::new(0.0, 100, 10, 0.1);
    let error = lyapunov_spectrum(&model.fields, &model.states, &[1.0], &config).unwrap_err();
    assert!(matches!(error, LyapunovError::InvalidConfig(_)));
}

#[test]
fn rejects_zero_steps() {
    let model = linear_decay_1d();
    let config = LyapunovConfig::new(0.01, 0, 10, 0.1);
    let error = lyapunov_spectrum(&model.fields, &model.states, &[1.0], &config).unwrap_err();
    assert!(matches!(error, LyapunovError::InvalidConfig(_)));
}

#[test]
fn rejects_empty_state_space() {
    let config = LyapunovConfig::default();
    let error = lyapunov_spectrum(&[], &[], &[], &config).unwrap_err();
    assert_eq!(error, LyapunovError::EmptyStateSpace);
}

#[test]
fn rejects_non_finite_initial_value() {
    let model = linear_decay_1d();
    let config = LyapunovConfig::default();
    let error = lyapunov_spectrum(&model.fields, &model.states, &[f64::NAN], &config).unwrap_err();
    assert!(matches!(error, LyapunovError::NonFiniteInput { .. }));
}
