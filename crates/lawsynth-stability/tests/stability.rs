//! Integration tests for `analyze_stability`.

mod common;

use std::f64::consts::PI;

use common::*;
use lawsynth_expr::Expr;
use lawsynth_stability::{Classification, StabilityConfig, StabilityError, analyze_stability};

fn box2(lower: f64, upper: f64) -> StabilityConfig {
    StabilityConfig::new(vec![(lower, upper), (lower, upper)])
}

#[test]
fn linear_stable_node_at_origin() {
    let (fields, states) = stable_node();
    let report = analyze_stability(&fields, &states, &box2(-1.0, 1.0)).unwrap();
    assert_eq!(report.fixed_points.len(), 1);
    let point = &report.fixed_points[0];
    assert!(close(&point.coordinates, &[0.0, 0.0], 1e-8));
    assert_eq!(point.classification, Classification::StableNode);
    // Eigenvalues of diag(-1, -2).
    let mut real_parts: Vec<f64> = point.eigenvalues.iter().map(|e| e.re).collect();
    real_parts.sort_by(|a, b| a.total_cmp(b));
    assert!((real_parts[0] + 2.0).abs() < 1e-9);
    assert!((real_parts[1] + 1.0).abs() < 1e-9);
}

#[test]
fn linear_center_at_origin() {
    let (fields, states) = center();
    let report = analyze_stability(&fields, &states, &box2(-1.0, 1.0)).unwrap();
    assert_eq!(report.fixed_points.len(), 1);
    assert!(close(&report.fixed_points[0].coordinates, &[0.0, 0.0], 1e-8));
    assert_eq!(report.fixed_points[0].classification, Classification::Center);
    assert!(report.fixed_points[0].classification.is_inconclusive());
}

#[test]
fn linear_saddle_at_origin() {
    let (fields, states) = saddle();
    let report = analyze_stability(&fields, &states, &box2(-1.0, 1.0)).unwrap();
    assert_eq!(report.fixed_points.len(), 1);
    assert_eq!(report.fixed_points[0].classification, Classification::Saddle);
}

#[test]
fn damped_oscillator_is_a_stable_spiral() {
    let (fields, states) = damped_oscillator();
    let report = analyze_stability(&fields, &states, &box2(-1.0, 1.0)).unwrap();
    assert_eq!(report.fixed_points.len(), 1);
    assert_eq!(report.fixed_points[0].classification, Classification::StableSpiral);
    // Eigenvalues -0.15 ± i·sqrt(1 - 0.15^2), so a genuine imaginary part.
    assert!(report.fixed_points[0].eigenvalues.iter().any(|e| e.im.abs() > 0.1));
}

#[test]
fn unstable_spiral_at_origin() {
    let (fields, states) = unstable_spiral();
    let report = analyze_stability(&fields, &states, &box2(-1.0, 1.0)).unwrap();
    assert_eq!(report.fixed_points.len(), 1);
    assert_eq!(report.fixed_points[0].classification, Classification::UnstableSpiral);
}

#[test]
fn pendulum_has_a_center_and_two_saddles() {
    let (fields, states) = pendulum();
    // Box wide enough to contain (0,0) and (±π, 0), resolution reaches near ±π.
    let config = box2(-4.0, 4.0).with_grid_resolution(5);
    let report = analyze_stability(&fields, &states, &config).unwrap();

    assert_eq!(report.fixed_points.len(), 3, "expected fixed points at -pi, 0, +pi");

    // Sorted lexicographically by coordinate: (-pi, 0), (0, 0), (pi, 0).
    let minus_pi = &report.fixed_points[0];
    let origin = &report.fixed_points[1];
    let plus_pi = &report.fixed_points[2];

    assert!(close(&minus_pi.coordinates, &[-PI, 0.0], 1e-6));
    assert_eq!(minus_pi.classification, Classification::Saddle);

    assert!(close(&origin.coordinates, &[0.0, 0.0], 1e-6));
    assert_eq!(origin.classification, Classification::Center);

    assert!(close(&plus_pi.coordinates, &[PI, 0.0], 1e-6));
    assert_eq!(plus_pi.classification, Classification::Saddle);
}

#[test]
fn lotka_volterra_has_a_saddle_and_an_interior_center() {
    let (fields, states) = lotka_volterra();
    let config = box2(-2.0, 3.0).with_grid_resolution(6);
    let report = analyze_stability(&fields, &states, &config).unwrap();

    assert_eq!(report.fixed_points.len(), 2);

    // Sorted lexicographically: (0, 0) then (1, 1).
    let origin = &report.fixed_points[0];
    let interior = &report.fixed_points[1];

    assert!(close(&origin.coordinates, &[0.0, 0.0], 1e-6));
    assert_eq!(origin.classification, Classification::Saddle);

    assert!(close(&interior.coordinates, &[1.0, 1.0], 1e-6));
    assert_eq!(interior.classification, Classification::Center);
}

#[test]
fn transcritical_normal_form_is_marginal() {
    let (fields, states) = transcritical();
    // Tight Newton tolerance drives the root close enough to 0 that the lone
    // eigenvalue 2·x* lands inside the (deliberately generous) marginal band.
    // The stiff root x* = ±sqrt(tol) is approached from both sides, so a
    // generous dedup radius merges the ± representatives into one fixed point.
    let config = StabilityConfig::new(vec![(-1.0, 1.0)])
        .with_tolerance(1e-12)
        .with_max_iterations(200)
        .with_dedup_tolerance(1e-3)
        .with_marginal_band(1e-2);
    let report = analyze_stability(&fields, &states, &config).unwrap();
    assert_eq!(report.fixed_points.len(), 1);
    assert!(close(&report.fixed_points[0].coordinates, &[0.0], 1e-4));
    assert_eq!(report.fixed_points[0].classification, Classification::Marginal);
    assert!(report.fixed_points[0].classification.is_inconclusive());
}

#[test]
fn no_fixed_point_in_box_reports_empty_with_seed_accounting() {
    let (fields, states) = no_fixed_point();
    let config = StabilityConfig::new(vec![(-3.0, 3.0)]);
    let report = analyze_stability(&fields, &states, &config).unwrap();
    assert!(report.is_empty());
    assert_eq!(report.fixed_points.len(), 0);
    assert_eq!(report.seeds_converged, 0, "no seed should converge to a non-existent root");
    assert!(report.seeds_total > 0);
}

#[test]
fn analysis_is_bit_identical_across_runs() {
    let (fields, states) = pendulum();
    let config = box2(-4.0, 4.0).with_grid_resolution(5);
    let first = analyze_stability(&fields, &states, &config).unwrap();
    let second = analyze_stability(&fields, &states, &config).unwrap();
    assert_eq!(first.to_canonical_string(), second.to_canonical_string());
    // And the reports themselves compare equal.
    assert_eq!(first, second);
}

#[test]
fn seed_accounting_is_reported() {
    let (fields, states) = stable_node();
    let report = analyze_stability(&fields, &states, &box2(-2.0, 2.0)).unwrap();
    assert!(report.seeds_total > 0);
    assert!(report.seeds_converged <= report.seeds_total);
    // Every seed of a globally attracting linear system converges to the origin.
    assert!(report.seeds_converged > 0);
}

#[test]
fn dimension_mismatch_is_rejected() {
    let (fields, states) = stable_node();
    // Two states, but a one-dimensional search box.
    let config = StabilityConfig::new(vec![(-1.0, 1.0)]);
    assert_eq!(
        analyze_stability(&fields, &states, &config),
        Err(StabilityError::DimensionMismatch { states: 2, search_box: 1 })
    );
}

#[test]
fn unknown_symbol_is_rejected() {
    // x' = a, where `a` is not a state: the field is not autonomous.
    let fields = vec![(id("x"), sym("a"))];
    let states = vec![id("x")];
    let config = StabilityConfig::new(vec![(-1.0, 1.0)]);
    assert_eq!(
        analyze_stability(&fields, &states, &config),
        Err(StabilityError::UnknownSymbol(id("a")))
    );
}

#[test]
fn missing_field_is_rejected() {
    // Two states declared, only one field supplied.
    let fields = vec![(id("x"), sym("y"))];
    let states = xy();
    let report = analyze_stability(&fields, &states, &box2(-1.0, 1.0));
    assert!(matches!(report, Err(StabilityError::Jacobian(_))));
}

#[test]
fn empty_state_space_is_rejected() {
    let fields: Vec<(lawsynth_core::Identifier, Expr)> = vec![];
    let states: Vec<lawsynth_core::Identifier> = vec![];
    let config = StabilityConfig::new(vec![]);
    assert_eq!(analyze_stability(&fields, &states, &config), Err(StabilityError::EmptyStateSpace));
}

#[test]
fn duplicate_state_is_rejected() {
    let fields = vec![(id("x"), neg(sym("x")))];
    let states = vec![id("x"), id("x")];
    let config = box2(-1.0, 1.0);
    assert!(matches!(
        analyze_stability(&fields, &states, &config),
        Err(StabilityError::Jacobian(_))
    ));
}
