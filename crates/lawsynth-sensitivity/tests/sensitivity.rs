//! Integration tests for forward sensitivity analysis.
//!
//! The suite pins two independent correctness anchors:
//!
//! 1. An **analytic check** against the closed-form sensitivity of a linear
//!    scalar law, to a tight tolerance.
//! 2. A **finite-difference cross-check** for nonlinear models (logistic and a
//!    2D Lotka–Volterra), comparing the integrated `∂x(t)/∂θ_j` against a central
//!    difference in parameter space, for every parameter at several times. This
//!    is the general proof that the variational integration is correct.
//!
//! Determinism and the typed error paths are exercised alongside.

mod common;

use common::{Model, finite_difference_sensitivity, id, logistic, lotka_volterra, simulate_state};
use lawsynth_core::Identifier;
use lawsynth_expr::{Expr, UnaryOperator};
use lawsynth_sensitivity::{SensitivityConfig, SensitivityError, forward_sensitivities};

/// Tolerance for the finite-difference cross-check (central difference, h = 1e-4).
const FD_TOLERANCE: f64 = 1e-6;

fn run(
    model: &Model,
    initial: &[f64],
    parameter_values: &[f64],
    config: &SensitivityConfig,
) -> Result<lawsynth_sensitivity::SensitivityTrajectory, SensitivityError> {
    forward_sensitivities(
        &model.fields,
        &model.states,
        &model.parameters,
        initial,
        parameter_values,
        config,
    )
}

#[test]
fn linear_scalar_sensitivity_matches_closed_form() {
    // ẋ = -theta * x has x(t) = x0 e^{-theta t} and ∂x/∂theta = -t x0 e^{-theta t}.
    let x = id("x");
    let theta = id("theta");
    let field = Expr::product(
        Expr::unary(UnaryOperator::Negate, Expr::symbol(theta.clone())),
        Expr::symbol(x.clone()),
    );
    let fields = vec![(x.clone(), field)];

    let x0 = 1.5;
    let theta_value = 0.7;
    let config = SensitivityConfig::new(0.0, 0.005, 200); // t up to 1.0
    let trajectory =
        forward_sensitivities(&fields, &[x], &[theta], &[x0], &[theta_value], &config).unwrap();

    for &step in &[40usize, 80, 120, 200] {
        let t = trajectory.times()[step];
        let expected = -t * x0 * (-theta_value * t).exp();
        let integrated = trajectory.partial(0, 0, step).unwrap();
        assert!(
            (integrated - expected).abs() < 1e-6,
            "t={t}: integrated {integrated} vs closed form {expected}"
        );
    }
}

#[test]
fn linear_scalar_state_matches_closed_form() {
    let x = id("x");
    let theta = id("theta");
    let field = Expr::product(
        Expr::unary(UnaryOperator::Negate, Expr::symbol(theta.clone())),
        Expr::symbol(x.clone()),
    );
    let fields = vec![(x.clone(), field)];

    let x0 = 1.5;
    let theta_value = 0.7;
    let config = SensitivityConfig::new(0.0, 0.005, 200);
    let trajectory =
        forward_sensitivities(&fields, &[x], &[theta], &[x0], &[theta_value], &config).unwrap();

    for &step in &[50usize, 120, 200] {
        let t = trajectory.times()[step];
        let expected = x0 * (-theta_value * t).exp();
        let integrated = trajectory.state_at(step).unwrap()[0];
        assert!((integrated - expected).abs() < 1e-9, "state mismatch at t={t}");
    }
}

#[test]
fn logistic_sensitivities_match_finite_difference() {
    let model = logistic();
    let initial = [0.2];
    let parameter_values = [0.8, 0.3];
    let dt = 0.005;
    let steps = 200;
    let config = SensitivityConfig::new(0.0, dt, steps);
    let trajectory = run(&model, &initial, &parameter_values, &config).unwrap();

    let h = 1e-4;
    for parameter in 0..model.parameters.len() {
        for &sub_steps in &[50usize, 100, 150, 200] {
            let reference = finite_difference_sensitivity(
                &model,
                &initial,
                &parameter_values,
                parameter,
                h,
                dt,
                sub_steps,
            );
            let integrated = trajectory.sensitivity_at(parameter, sub_steps).unwrap();
            for (component, (&num, &analytic)) in reference.iter().zip(integrated).enumerate() {
                assert!(
                    (num - analytic).abs() < FD_TOLERANCE,
                    "logistic dparam {parameter} dstate {component} at step {sub_steps}: \
                     fd {num} vs integrated {analytic}"
                );
            }
        }
    }
}

#[test]
fn lotka_volterra_sensitivities_match_finite_difference() {
    let model = lotka_volterra();
    let initial = [1.0, 1.0];
    let parameter_values = [1.5, 1.0, 3.0, 1.0]; // a, b, c, d
    let dt = 0.005;
    let steps = 200;
    let config = SensitivityConfig::new(0.0, dt, steps);
    let trajectory = run(&model, &initial, &parameter_values, &config).unwrap();

    let h = 1e-4;
    for parameter in 0..model.parameters.len() {
        for &sub_steps in &[50usize, 100, 150, 200] {
            let reference = finite_difference_sensitivity(
                &model,
                &initial,
                &parameter_values,
                parameter,
                h,
                dt,
                sub_steps,
            );
            let integrated = trajectory.sensitivity_at(parameter, sub_steps).unwrap();
            for (component, (&num, &analytic)) in reference.iter().zip(integrated).enumerate() {
                assert!(
                    (num - analytic).abs() < FD_TOLERANCE,
                    "lotka-volterra dparam {parameter} dstate {component} at step {sub_steps}: \
                     fd {num} vs integrated {analytic}"
                );
            }
        }
    }
}

#[test]
fn initial_sensitivity_is_zero() {
    let model = lotka_volterra();
    let config = SensitivityConfig::new(0.0, 0.01, 10);
    let trajectory = run(&model, &[1.0, 1.0], &[1.5, 1.0, 3.0, 1.0], &config).unwrap();
    for parameter in 0..model.parameters.len() {
        for &value in trajectory.sensitivity_at(parameter, 0).unwrap() {
            assert_eq!(value, 0.0, "S_{parameter}(0) must be exactly zero");
        }
    }
}

#[test]
fn integration_is_bit_identical_across_runs() {
    let model = logistic();
    let initial = [0.25];
    let parameter_values = [0.9, 0.4];
    let config = SensitivityConfig::new(0.0, 0.01, 137);

    let first = run(&model, &initial, &parameter_values, &config).unwrap();
    let second = run(&model, &initial, &parameter_values, &config).unwrap();

    assert_eq!(first.to_canonical_string(), second.to_canonical_string());
    // Spot-check the raw bits too, not just the fingerprint.
    for step in 0..first.sample_count() {
        for parameter in 0..model.parameters.len() {
            let a = first.partial(0, parameter, step).unwrap();
            let b = second.partial(0, parameter, step).unwrap();
            assert_eq!(a.to_bits(), b.to_bits());
        }
    }
}

#[test]
fn parameter_absent_from_fields_has_zero_sensitivity() {
    // Logistic with an extra parameter that appears nowhere in the fields. Its
    // partial ∂f/∂phantom is identically zero, so its sensitivity stays zero for
    // all time — the honest, non-fabricated answer.
    let mut model = logistic();
    let phantom = id("phantom");
    model.parameters.push(phantom);

    let config = SensitivityConfig::new(0.0, 0.01, 50);
    let trajectory = run(&model, &[0.2], &[0.8, 0.3, 5.0], &config).unwrap();

    let phantom_index = model.parameters.len() - 1;
    for step in 0..trajectory.sample_count() {
        for &value in trajectory.sensitivity_at(phantom_index, step).unwrap() {
            assert_eq!(value, 0.0, "an absent parameter must have zero sensitivity");
        }
    }
    // A parameter that IS present should be nonzero somewhere, as a sanity guard.
    let present = trajectory.partial(0, 0, trajectory.sample_count() - 1).unwrap();
    assert!(present.abs() > 0.0);
}

#[test]
fn unknown_field_symbol_is_rejected() {
    // The field references `k`, which is neither a state nor a declared parameter.
    let x = id("x");
    let theta = id("theta");
    let k = id("k");
    let field = Expr::product(
        Expr::sum(Expr::symbol(theta.clone()), Expr::symbol(k.clone())),
        Expr::symbol(x.clone()),
    );
    let fields = vec![(x.clone(), field)];
    let config = SensitivityConfig::new(0.0, 0.01, 10);

    let error =
        forward_sensitivities(&fields, &[x], &[theta], &[1.0], &[0.5], &config).unwrap_err();
    assert_eq!(error, SensitivityError::UnknownSymbol(k));
}

#[test]
fn state_dimension_mismatch_is_rejected() {
    let model = logistic();
    let config = SensitivityConfig::new(0.0, 0.01, 10);
    let error = run(&model, &[0.2, 0.3], &[0.8, 0.3], &config).unwrap_err();
    assert_eq!(error, SensitivityError::StateDimensionMismatch { states: 1, initial: 2 });
}

#[test]
fn parameter_dimension_mismatch_is_rejected() {
    let model = logistic();
    let config = SensitivityConfig::new(0.0, 0.01, 10);
    let error = run(&model, &[0.2], &[0.8], &config).unwrap_err();
    assert_eq!(error, SensitivityError::ParameterDimensionMismatch { parameters: 2, values: 1 });
}

#[test]
fn duplicate_parameter_is_rejected() {
    let x = id("x");
    let theta = id("theta");
    let field = Expr::product(Expr::symbol(theta.clone()), Expr::symbol(x.clone()));
    let fields = vec![(x.clone(), field)];
    let config = SensitivityConfig::new(0.0, 0.01, 10);

    let error = forward_sensitivities(
        &fields,
        &[x],
        &[theta.clone(), theta.clone()],
        &[1.0],
        &[0.5, 0.5],
        &config,
    )
    .unwrap_err();
    assert_eq!(error, SensitivityError::DuplicateParameter(theta));
}

#[test]
fn parameter_that_is_also_a_state_is_rejected() {
    let x = id("x");
    let field = Expr::product(Expr::symbol(x.clone()), Expr::symbol(x.clone()));
    let fields = vec![(x.clone(), field)];
    let config = SensitivityConfig::new(0.0, 0.01, 10);

    let states = std::slice::from_ref(&x);
    let error =
        forward_sensitivities(&fields, states, states, &[1.0], &[0.5], &config).unwrap_err();
    assert_eq!(error, SensitivityError::ParameterIsState(x));
}

#[test]
fn empty_state_space_is_rejected() {
    let config = SensitivityConfig::new(0.0, 0.01, 10);
    let fields: Vec<(Identifier, Expr)> = Vec::new();
    let error = forward_sensitivities(&fields, &[], &[], &[], &[], &config).unwrap_err();
    assert_eq!(error, SensitivityError::EmptyStateSpace);
}

#[test]
fn invalid_config_is_rejected() {
    let model = logistic();
    let config = SensitivityConfig::new(0.0, 0.0, 10); // dt = 0
    let error = run(&model, &[0.2], &[0.8, 0.3], &config).unwrap_err();
    assert!(matches!(error, SensitivityError::InvalidConfig(_)));
}

#[test]
fn missing_field_is_rejected() {
    // Two states declared but only one field supplied: analytic_jacobian rejects
    // it and the error is surfaced as a Jacobian error.
    let x = id("x");
    let y = id("y");
    let fields = vec![(x.clone(), Expr::symbol(x.clone()))];
    let config = SensitivityConfig::new(0.0, 0.01, 10);
    let error =
        forward_sensitivities(&fields, &[x, y], &[], &[1.0, 1.0], &[], &config).unwrap_err();
    assert!(matches!(error, SensitivityError::Jacobian(_)));
}

#[test]
fn partial_helper_reports_out_of_range() {
    let model = logistic();
    let config = SensitivityConfig::new(0.0, 0.01, 5);
    let trajectory = run(&model, &[0.2], &[0.8, 0.3], &config).unwrap();
    assert!(trajectory.partial(0, 0, 999).is_none());
    assert!(trajectory.partial(9, 0, 0).is_none());
    assert!(trajectory.partial(0, 9, 0).is_none());
    assert!(trajectory.sensitivity_at(0, 999).is_none());
    assert!(trajectory.state_at(999).is_none());
}

#[test]
fn nonlinear_state_trajectory_is_reproducible_against_reference() {
    // The crate's own state trajectory must match the standalone RK4 reference
    // used by the finite-difference cross-check, confirming the augmented system
    // integrates the state block identically.
    let model = logistic();
    let initial = [0.2];
    let parameter_values = [0.8, 0.3];
    let dt = 0.01;
    let steps = 100;
    let config = SensitivityConfig::new(0.0, dt, steps);
    let trajectory = run(&model, &initial, &parameter_values, &config).unwrap();

    let reference = simulate_state(&model, &initial, &parameter_values, dt, steps);
    let integrated = trajectory.state_at(steps).unwrap();
    assert_eq!(integrated[0].to_bits(), reference[0].to_bits());
}
