//! Integration tests for forecast-uncertainty propagation.
//!
//! The suite pins several independent correctness anchors:
//!
//! 1. The delta variance against the **closed-form** delta variance of a linear
//!    scalar law, to a tight tolerance.
//! 2. **Delta ≈ Monte-Carlo** for small parameter uncertainty on a nonlinear
//!    model, and the honest **divergence** under large uncertainty.
//! 3. Monotonicity (bands widen with time and with larger `Cov(θ)`), coverage
//!    sanity, an end-to-end pipeline from a bootstrap ensemble, bit-identical
//!    determinism, and the typed error paths.

mod common;

use common::{id, linear_decay, linear_growth, logistic, poly_growth};
use lawsynth_expr::{Expr, UnaryOperator};
use lawsynth_propagate::{
    EnsembleSource, PropagateError, covariance_from_ensemble, delta_forecast, monte_carlo_forecast,
    z_for_confidence,
};
use lawsynth_sensitivity::{SensitivityConfig, SensitivityError};
use lawsynth_uncertainty::{BootstrapCoefficientConfig, ResampleMode, bootstrap_coefficients};

const DT: f64 = 0.01;
const STEPS: usize = 100;

fn config() -> SensitivityConfig {
    SensitivityConfig::new(0.0, DT, STEPS)
}

// ----------------------------------------------------------------------------
// 1. Delta method vs closed-form analytic variance.
// ----------------------------------------------------------------------------

#[test]
fn delta_variance_matches_closed_form_linear_decay() {
    // ẋ = -θ x, x(t) = x0 e^{-θt}, ∂x/∂θ = -t x0 e^{-θt}.
    // With Var(θ) = s², the delta variance is (t x0 e^{-θt})² s².
    let model = linear_decay();
    let x0 = 1.5;
    let theta = 0.7;
    let variance_theta = 0.04; // s² = 0.04, s = 0.2
    let cov = vec![vec![variance_theta]];
    let z = 1.0;

    let bands = delta_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[x0],
        &[theta],
        &cov,
        &config(),
        z,
    )
    .unwrap();

    for &step in &[10usize, 40, 70, 100] {
        let t = bands.times()[step];
        let sensitivity = -t * x0 * (-theta * t).exp();
        let expected_variance = sensitivity * sensitivity * variance_theta;
        let got_variance = bands.variance()[0][step];
        assert!(
            (got_variance - expected_variance).abs() < 1e-9,
            "t={t}: delta variance {got_variance} vs closed form {expected_variance}"
        );
        // Mean is the nominal trajectory, band is symmetric around it.
        let expected_mean = x0 * (-theta * t).exp();
        assert!((bands.mean()[0][step] - expected_mean).abs() < 1e-9);
        let half = z * expected_variance.sqrt();
        assert!((bands.upper()[0][step] - (expected_mean + half)).abs() < 1e-9);
        assert!((bands.lower()[0][step] - (expected_mean - half)).abs() < 1e-9);
    }
}

#[test]
fn delta_variance_scales_quadratically_with_parameter_std() {
    // Doubling the parameter standard deviation (4x the variance) quadruples the
    // state variance and doubles the band half-width — a linear-map property.
    let model = linear_decay();
    let base = vec![vec![0.01]];
    let quad = vec![vec![0.04]];
    let cfg = config();

    let a = delta_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[1.0],
        &[0.5],
        &base,
        &cfg,
        1.96,
    )
    .unwrap();
    let b = delta_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[1.0],
        &[0.5],
        &quad,
        &cfg,
        1.96,
    )
    .unwrap();

    for step in 1..=STEPS {
        let va = a.variance()[0][step];
        let vb = b.variance()[0][step];
        if va > 1e-12 {
            assert!((vb / va - 4.0).abs() < 1e-9, "variance ratio at step {step} = {}", vb / va);
        }
    }
}

// ----------------------------------------------------------------------------
// 2. Monotonicity.
// ----------------------------------------------------------------------------

#[test]
fn delta_bands_widen_with_time_for_growing_sensitivity() {
    // ẋ = θ x has ∂x/∂θ = t x0 e^{θt}, strictly increasing in t, so the delta
    // variance and band width strictly increase.
    let model = linear_growth();
    let cov = vec![vec![0.01]];
    let bands = delta_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[1.0],
        &[0.6],
        &cov,
        &config(),
        1.96,
    )
    .unwrap();

    for step in 1..=STEPS {
        let prev = bands.band_width(0, step - 1).unwrap();
        let curr = bands.band_width(0, step).unwrap();
        assert!(curr > prev, "band width not increasing at step {step}: {prev} -> {curr}");
    }
}

#[test]
fn delta_bands_widen_with_larger_covariance() {
    let model = logistic();
    let cfg = config();
    let small = vec![vec![1e-4, 0.0], vec![0.0, 1e-4]];
    let large = vec![vec![4e-4, 0.0], vec![0.0, 4e-4]];

    let a = delta_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        &[0.8, 0.3],
        &small,
        &cfg,
        1.96,
    )
    .unwrap();
    let b = delta_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        &[0.8, 0.3],
        &large,
        &cfg,
        1.96,
    )
    .unwrap();

    for step in 1..=STEPS {
        let wa = a.band_width(0, step).unwrap();
        let wb = b.band_width(0, step).unwrap();
        if wa > 1e-9 {
            assert!(wb > wa, "larger covariance did not widen band at step {step}");
        }
    }
}

// ----------------------------------------------------------------------------
// 3. Delta vs Monte-Carlo: agreement (small σ) and divergence (large σ).
// ----------------------------------------------------------------------------

#[test]
fn delta_and_monte_carlo_agree_for_small_uncertainty() {
    let model = logistic();
    let cfg = config();
    let mean = [0.8, 0.3];
    let cov = vec![vec![4e-5, 1e-5], vec![1e-5, 4e-5]];
    let confidence = 0.95;
    let z = z_for_confidence(confidence).unwrap();

    let delta = delta_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        &mean,
        &cov,
        &cfg,
        z,
    )
    .unwrap();
    let mc = monte_carlo_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        EnsembleSource::Gaussian { mean: &mean, covariance: &cov },
        &cfg,
        8_000,
        0xABCD_1234,
        confidence,
    )
    .unwrap();

    for &step in &[40usize, 70, 100] {
        let delta_var = delta.variance()[0][step];
        let mc_var = mc.variance()[0][step];
        assert!(delta_var > 0.0);
        // Variances agree to a few percent in the small-σ limit.
        let relative = (mc_var - delta_var).abs() / delta_var;
        assert!(
            relative < 0.10,
            "step {step}: delta var {delta_var} vs mc var {mc_var} (rel {relative})"
        );

        // Band widths agree too (looser: percentile estimates carry sampling noise).
        let delta_width = delta.band_width(0, step).unwrap();
        let mc_width = mc.band_width(0, step).unwrap();
        let width_rel = (mc_width - delta_width).abs() / delta_width;
        assert!(width_rel < 0.10, "step {step}: delta width {delta_width} vs mc width {mc_width}");
    }
}

#[test]
fn delta_and_monte_carlo_diverge_for_large_uncertainty() {
    // Large parameter uncertainty makes the nonlinear logistic response skewed,
    // so the symmetric first-order delta band no longer matches the empirical
    // Monte-Carlo band. This is the honest limitation of the delta method.
    let model = logistic();
    let cfg = config();
    let mean = [0.8, 0.3];
    let cov = vec![vec![0.09, 0.0], vec![0.0, 0.02]];
    let confidence = 0.95;
    let z = z_for_confidence(confidence).unwrap();

    let delta = delta_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        &mean,
        &cov,
        &cfg,
        z,
    )
    .unwrap();
    let mc = monte_carlo_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        EnsembleSource::Gaussian { mean: &mean, covariance: &cov },
        &cfg,
        8_000,
        0x5151_5151,
        confidence,
    )
    .unwrap();

    let step = STEPS;
    let delta_width = delta.band_width(0, step).unwrap();
    let mc_width = mc.band_width(0, step).unwrap();
    let width_rel = (mc_width - delta_width).abs() / delta_width;
    assert!(width_rel > 0.05, "expected divergence at large σ, got rel width diff {width_rel}");

    // The Monte-Carlo band is visibly asymmetric about its mean; the delta band
    // is exactly symmetric by construction.
    let mc_mean = mc.mean()[0][step];
    let up = mc.upper()[0][step] - mc_mean;
    let down = mc_mean - mc.lower()[0][step];
    let asymmetry = (up - down).abs() / (up + down);
    assert!(asymmetry > 0.02, "expected asymmetric MC band, got asymmetry {asymmetry}");
}

// ----------------------------------------------------------------------------
// 4. Monte-Carlo coverage sanity.
// ----------------------------------------------------------------------------

#[test]
fn monte_carlo_band_contains_nominal_trajectory() {
    let model = logistic();
    let cfg = config();
    let mean = [0.8, 0.3];
    let cov = vec![vec![1e-3, 0.0], vec![0.0, 1e-3]];

    // The nominal (true-at-mean-θ) trajectory is the delta mean.
    let nominal = delta_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        &mean,
        &cov,
        &cfg,
        1.96,
    )
    .unwrap();
    let mc = monte_carlo_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        EnsembleSource::Gaussian { mean: &mean, covariance: &cov },
        &cfg,
        8_000,
        0xFACE,
        0.95,
    )
    .unwrap();

    for step in 0..=STEPS {
        let x = nominal.mean()[0][step];
        let lo = mc.lower()[0][step];
        let hi = mc.upper()[0][step];
        assert!(lo <= x && x <= hi, "nominal {x} outside MC band [{lo}, {hi}] at step {step}");
    }
}

#[test]
fn higher_confidence_widens_monte_carlo_band() {
    let model = logistic();
    let cfg = config();
    let mean = [0.8, 0.3];
    let cov = vec![vec![1e-3, 0.0], vec![0.0, 1e-3]];
    let source = || EnsembleSource::Gaussian { mean: &mean, covariance: &cov };

    let narrow = monte_carlo_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        source(),
        &cfg,
        8_000,
        7,
        0.80,
    )
    .unwrap();
    let wide = monte_carlo_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        source(),
        &cfg,
        8_000,
        7,
        0.99,
    )
    .unwrap();

    for &step in &[50usize, 80, 100] {
        assert!(
            wide.band_width(0, step).unwrap() > narrow.band_width(0, step).unwrap(),
            "99% band not wider than 80% at step {step}"
        );
    }
}

#[test]
fn monte_carlo_band_widens_with_larger_covariance() {
    let model = logistic();
    let cfg = config();
    let mean = [0.8, 0.3];
    let small = vec![vec![1e-4, 0.0], vec![0.0, 1e-4]];
    let large = vec![vec![9e-4, 0.0], vec![0.0, 9e-4]];

    let a = monte_carlo_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        EnsembleSource::Gaussian { mean: &mean, covariance: &small },
        &cfg,
        8_000,
        3,
        0.95,
    )
    .unwrap();
    let b = monte_carlo_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        EnsembleSource::Gaussian { mean: &mean, covariance: &large },
        &cfg,
        8_000,
        3,
        0.95,
    )
    .unwrap();

    for &step in &[50usize, 80, 100] {
        assert!(
            b.band_width(0, step).unwrap() > a.band_width(0, step).unwrap(),
            "larger covariance did not widen MC band at step {step}"
        );
    }
}

// ----------------------------------------------------------------------------
// 5. End-to-end from a bootstrap ensemble.
// ----------------------------------------------------------------------------

#[test]
fn end_to_end_from_bootstrap_ensemble() {
    // Synthetic library [x, x²] with target 0.8 x - 0.3 x² plus small structured
    // noise, so the coefficient bootstrap has a genuine spread.
    let mut theta = Vec::new();
    let mut target = Vec::new();
    for i in 0..60 {
        let x = 0.1 + 0.03 * i as f64;
        theta.push(vec![x, x * x]);
        let noise = 0.01 * ((i as f64) * 0.7).sin();
        target.push(0.8 * x - 0.3 * x * x + noise);
    }
    let boot_config = BootstrapCoefficientConfig {
        resamples: 128,
        seed: 0x1234_5678,
        confidence: 0.95,
        mode: ResampleMode::Cases,
        ..BootstrapCoefficientConfig::default()
    };
    let ensemble = bootstrap_coefficients(&theta, &target, &boot_config).unwrap();

    // Derive Cov(θ) from the replicate coefficient vectors.
    let cov = covariance_from_ensemble(&ensemble);
    assert_eq!(cov.len(), 2);
    assert!(cov[0][0] > 0.0 && cov[1][1] > 0.0);

    // The discovered coefficients (means) map sign-for-sign onto poly_growth.
    let mean_params: Vec<f64> = ensemble.terms.iter().map(|term| term.mean).collect();
    let model = poly_growth();
    let cfg = config();
    let z = z_for_confidence(0.95).unwrap();

    // Delta bands straight from the bootstrap covariance.
    let delta = delta_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        &mean_params,
        &cov,
        &cfg,
        z,
    )
    .unwrap();
    assert_eq!(delta.sample_count(), STEPS + 1);
    // A late-time band has positive width — uncertainty genuinely propagated.
    assert!(delta.band_width(0, STEPS).unwrap() > 0.0);

    // Monte-Carlo directly resampling the bootstrap replicate coefficients.
    let mc = monte_carlo_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        EnsembleSource::Replicates { draws: &ensemble.replicates },
        &cfg,
        4_000,
        0x99,
        0.95,
    )
    .unwrap();
    assert!(mc.band_width(0, STEPS).unwrap() > 0.0);
    // The delta mean (nominal trajectory) lies inside the resampled MC band.
    for &step in &[50usize, 80, 100] {
        let x = delta.mean()[0][step];
        assert!(mc.lower()[0][step] <= x && x <= mc.upper()[0][step]);
    }
}

// ----------------------------------------------------------------------------
// 6. Determinism.
// ----------------------------------------------------------------------------

#[test]
fn delta_forecast_is_bit_identical() {
    let model = logistic();
    let cfg = config();
    let cov = vec![vec![1e-3, 2e-4], vec![2e-4, 1e-3]];
    let first = delta_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.25],
        &[0.9, 0.4],
        &cov,
        &cfg,
        1.96,
    )
    .unwrap();
    let second = delta_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.25],
        &[0.9, 0.4],
        &cov,
        &cfg,
        1.96,
    )
    .unwrap();
    assert_eq!(first.to_canonical_string(), second.to_canonical_string());
    for step in 0..first.sample_count() {
        assert_eq!(first.variance()[0][step].to_bits(), second.variance()[0][step].to_bits());
    }
}

#[test]
fn monte_carlo_forecast_is_bit_identical() {
    let model = logistic();
    let cfg = config();
    let mean = [0.8, 0.3];
    let cov = vec![vec![1e-3, 0.0], vec![0.0, 1e-3]];
    let run = || {
        monte_carlo_forecast(
            &model.fields,
            &model.states,
            &model.parameters,
            &[0.2],
            EnsembleSource::Gaussian { mean: &mean, covariance: &cov },
            &cfg,
            2_000,
            0xDEAD_BEEF,
            0.95,
        )
        .unwrap()
    };
    assert_eq!(run().to_canonical_string(), run().to_canonical_string());
}

// ----------------------------------------------------------------------------
// 7. Error paths.
// ----------------------------------------------------------------------------

#[test]
fn delta_rejects_covariance_dimension_mismatch() {
    let model = logistic(); // two parameters
    let cov = vec![vec![1e-3]]; // 1x1
    let error = delta_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        &[0.8, 0.3],
        &cov,
        &config(),
        1.96,
    )
    .unwrap_err();
    assert_eq!(error, PropagateError::CovarianceDimensionMismatch { expected: 2, actual: 1 });
}

#[test]
fn delta_rejects_non_square_covariance() {
    let model = linear_decay();
    let cov = vec![vec![1e-3, 0.0]]; // 1 row of width 2, expected 1x1
    let error = delta_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[1.0],
        &[0.5],
        &cov,
        &config(),
        1.96,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        PropagateError::CovarianceDimensionMismatch { .. }
            | PropagateError::CovarianceNotSquare { .. }
    ));
}

#[test]
fn delta_rejects_indefinite_covariance() {
    // ẋ = -(p1 + p2) x has equal sensitivities to p1 and p2. With the indefinite
    // covariance [[1,-2],[-2,1]] the quadratic form s(1)-2s(2)+... is negative,
    // so the propagated variance would be negative — reported, not fabricated.
    let x = id("x");
    let p1 = id("p1");
    let p2 = id("p2");
    let sum = Expr::sum(Expr::symbol(p1.clone()), Expr::symbol(p2.clone()));
    let field = Expr::product(Expr::unary(UnaryOperator::Negate, sum), Expr::symbol(x.clone()));
    let fields = vec![(x.clone(), field)];
    let states = vec![x];
    let parameters = vec![p1, p2];
    let cov = vec![vec![1.0, -2.0], vec![-2.0, 1.0]];

    let error =
        delta_forecast(&fields, &states, &parameters, &[1.0], &[0.5, 0.5], &cov, &config(), 1.96)
            .unwrap_err();
    assert_eq!(error, PropagateError::NotPositiveSemiDefinite);
}

#[test]
fn delta_rejects_non_finite_multiplier() {
    let model = linear_decay();
    let cov = vec![vec![1e-3]];
    let error = delta_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[1.0],
        &[0.5],
        &cov,
        &config(),
        f64::NAN,
    )
    .unwrap_err();
    assert_eq!(error, PropagateError::NonFiniteMultiplier);
}

#[test]
fn delta_propagates_sensitivity_errors() {
    // Field references an undeclared symbol; the sensitivity layer rejects it.
    let x = id("x");
    let theta = id("theta");
    let k = id("k");
    let field = Expr::product(
        Expr::sum(Expr::symbol(theta.clone()), Expr::symbol(k.clone())),
        Expr::symbol(x.clone()),
    );
    let fields = vec![(x.clone(), field)];
    let cov = vec![vec![1e-3]];
    let error =
        delta_forecast(&fields, &[x], &[theta], &[1.0], &[0.5], &cov, &config(), 1.96).unwrap_err();
    assert_eq!(error, PropagateError::Sensitivity(SensitivityError::UnknownSymbol(k)));
}

#[test]
fn monte_carlo_rejects_zero_samples() {
    let model = logistic();
    let mean = [0.8, 0.3];
    let cov = vec![vec![1e-3, 0.0], vec![0.0, 1e-3]];
    let error = monte_carlo_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        EnsembleSource::Gaussian { mean: &mean, covariance: &cov },
        &config(),
        0,
        1,
        0.95,
    )
    .unwrap_err();
    assert_eq!(error, PropagateError::ZeroSamples);
}

#[test]
fn monte_carlo_rejects_invalid_confidence() {
    let model = logistic();
    let mean = [0.8, 0.3];
    let cov = vec![vec![1e-3, 0.0], vec![0.0, 1e-3]];
    let error = monte_carlo_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        EnsembleSource::Gaussian { mean: &mean, covariance: &cov },
        &config(),
        100,
        1,
        1.5,
    )
    .unwrap_err();
    assert_eq!(error, PropagateError::InvalidConfidence(1.5));
}

#[test]
fn monte_carlo_rejects_replicate_dimension_mismatch() {
    let model = logistic(); // two parameters
    let draws = vec![vec![0.8, 0.3], vec![0.7]]; // second replicate too short
    let error = monte_carlo_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        EnsembleSource::Replicates { draws: &draws },
        &config(),
        100,
        1,
        0.95,
    )
    .unwrap_err();
    assert_eq!(error, PropagateError::ReplicateDimensionMismatch { expected: 2, actual: 1 });
}

#[test]
fn monte_carlo_rejects_empty_replicate_ensemble() {
    let model = logistic();
    let draws: Vec<Vec<f64>> = Vec::new();
    let error = monte_carlo_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        EnsembleSource::Replicates { draws: &draws },
        &config(),
        100,
        1,
        0.95,
    )
    .unwrap_err();
    assert_eq!(error, PropagateError::EmptyEnsemble);
}

#[test]
fn monte_carlo_rejects_indefinite_gaussian_covariance() {
    let model = logistic();
    let mean = [0.8, 0.3];
    let cov = vec![vec![1.0, 2.0], vec![2.0, 1.0]]; // indefinite
    let error = monte_carlo_forecast(
        &model.fields,
        &model.states,
        &model.parameters,
        &[0.2],
        EnsembleSource::Gaussian { mean: &mean, covariance: &cov },
        &config(),
        100,
        1,
        0.95,
    )
    .unwrap_err();
    assert_eq!(error, PropagateError::NotPositiveSemiDefinite);
}
