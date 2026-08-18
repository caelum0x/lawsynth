//! Integration tests on textbook parameterized systems with known bifurcations.
//!
//! Each analytic critical value is `mu = 0`; we assert the detected value lands
//! within a stated tolerance, the kind (Fold vs Hopf) is correct, and the
//! fixed-point structure changes as the theory predicts. Determinism and the
//! error paths are exercised too.

mod common;

use lawsynth_bifurcation::{
    BifurcationKind, Classification, ContinuationReport, StabilityConfig, Sweep, continuation,
};
use lawsynth_core::Identifier;
use lawsynth_expr::Expr;

/// Tolerance on the localized critical parameter versus the analytic `mu = 0`.
const CRITICAL_TOLERANCE: f64 = 1e-2;

/// A 1D sweep whose match tolerance comfortably exceeds the per-step coordinate
/// motion of the textbook branches.
fn sweep_1d() -> Sweep {
    Sweep::new(-1.0, 1.0, 21).with_match_tolerance(0.3)
}

fn config_1d() -> StabilityConfig {
    StabilityConfig::new(vec![(-2.0, 2.0)])
}

/// The fixed point in a sample whose first coordinate is nearest `target`.
fn classification_near(
    report: &ContinuationReport,
    sample_index: usize,
    target: f64,
) -> Classification {
    report.samples[sample_index]
        .report
        .fixed_points
        .iter()
        .min_by(|a, b| {
            (a.coordinates[0] - target).abs().total_cmp(&(b.coordinates[0] - target).abs())
        })
        .expect("sample has a fixed point")
        .classification
}

#[test]
fn saddle_node_detects_a_fold_near_zero() {
    let (fields, states, parameter) = common::saddle_node();
    let report = continuation(&fields, &states, &parameter, &sweep_1d(), &config_1d()).unwrap();

    assert_eq!(report.bifurcation_count(), 1, "one fold expected");
    let fold = &report.bifurcations[0];
    assert_eq!(fold.kind, BifurcationKind::Fold);
    assert!(fold.parameter_value.abs() < CRITICAL_TOLERANCE, "mu* = {}", fold.parameter_value);
    // The crossing eigenvalue is (near-)real at a fold.
    assert!(fold.eigenvalue.im.abs() < 1e-6);
}

#[test]
fn saddle_node_fixed_point_count_changes_across_the_fold() {
    let (fields, states, parameter) = common::saddle_node();
    let report = continuation(&fields, &states, &parameter, &sweep_1d(), &config_1d()).unwrap();

    // mu = -1: no real fixed point; mu = +1: two (x = ±1).
    assert_eq!(report.samples[0].report.fixed_points.len(), 0);
    assert_eq!(report.samples[20].report.fixed_points.len(), 2);
}

#[test]
fn transcritical_detects_a_fold_near_zero() {
    let (fields, states, parameter) = common::transcritical();
    let report = continuation(&fields, &states, &parameter, &sweep_1d(), &config_1d()).unwrap();

    assert_eq!(report.bifurcation_count(), 1);
    let fold = &report.bifurcations[0];
    assert_eq!(fold.kind, BifurcationKind::Fold);
    assert!(fold.parameter_value.abs() < CRITICAL_TOLERANCE, "mu* = {}", fold.parameter_value);
}

#[test]
fn transcritical_branches_exchange_stability_at_zero() {
    let (fields, states, parameter) = common::transcritical();
    let report = continuation(&fields, &states, &parameter, &sweep_1d(), &config_1d()).unwrap();

    // The x = 0 branch is stable for mu < 0 and unstable for mu > 0.
    assert_eq!(classification_near(&report, 5, 0.0), Classification::StableNode); // mu = -0.5
    assert_eq!(classification_near(&report, 15, 0.0), Classification::UnstableNode); // mu = +0.5
    // The x = mu branch does the opposite.
    assert_eq!(classification_near(&report, 5, -0.5), Classification::UnstableNode);
    assert_eq!(classification_near(&report, 15, 0.5), Classification::StableNode);
}

#[test]
fn pitchfork_detects_a_fold_near_zero() {
    let (fields, states, parameter) = common::pitchfork();
    let report = continuation(&fields, &states, &parameter, &sweep_1d(), &config_1d()).unwrap();

    assert_eq!(report.bifurcation_count(), 1);
    let fold = &report.bifurcations[0];
    assert_eq!(fold.kind, BifurcationKind::Fold);
    assert!(fold.parameter_value.abs() < CRITICAL_TOLERANCE, "mu* = {}", fold.parameter_value);
}

#[test]
fn pitchfork_one_branch_becomes_three() {
    let (fields, states, parameter) = common::pitchfork();
    let report = continuation(&fields, &states, &parameter, &sweep_1d(), &config_1d()).unwrap();

    // mu = -0.5: only x = 0; mu = +0.5: x = 0 and ±√0.5.
    assert_eq!(report.samples[5].report.fixed_points.len(), 1);
    assert_eq!(report.samples[15].report.fixed_points.len(), 3);
}

#[test]
fn hopf_detects_a_complex_pair_crossing_near_zero() {
    let (fields, states, parameter) = common::hopf();
    let sweep = Sweep::new(-1.0, 1.0, 21);
    let config = StabilityConfig::new(vec![(-1.5, 1.5), (-1.5, 1.5)]);
    let report = continuation(&fields, &states, &parameter, &sweep, &config).unwrap();

    assert_eq!(report.bifurcation_count(), 1);
    let hopf = &report.bifurcations[0];
    assert_eq!(hopf.kind, BifurcationKind::Hopf);
    assert!(hopf.parameter_value.abs() < CRITICAL_TOLERANCE, "mu* = {}", hopf.parameter_value);
    // The crossing eigenvalue is genuinely complex (Im ≈ ±1).
    assert!((hopf.eigenvalue.im.abs() - 1.0).abs() < 1e-3, "im = {}", hopf.eigenvalue.im);
    assert!(hopf.eigenvalue.re.abs() < 1e-3, "re = {}", hopf.eigenvalue.re);
}

#[test]
fn hopf_origin_persists_as_a_single_branch() {
    let (fields, states, parameter) = common::hopf();
    let sweep = Sweep::new(-1.0, 1.0, 21);
    let config = StabilityConfig::new(vec![(-1.5, 1.5), (-1.5, 1.5)]);
    let report = continuation(&fields, &states, &parameter, &sweep, &config).unwrap();

    // Exactly one fixed point (the origin) at every parameter value.
    for sample in &report.samples {
        assert_eq!(sample.report.fixed_points.len(), 1);
    }
    assert_eq!(report.branch_count(), 1);
    assert_eq!(report.branches[0].points.len(), 21);
}

#[test]
fn a_monotone_stable_system_has_no_bifurcation() {
    // x' = mu - x: a single stable fixed point x = mu (eigenvalue -1) for all mu.
    let x = common::id("x");
    let mu = common::id("mu");
    let field = Expr::difference(Expr::symbol(mu.clone()), Expr::symbol(x.clone()));
    let fields = vec![(x.clone(), field)];
    let report = continuation(&fields, &[x], &mu, &sweep_1d(), &config_1d()).unwrap();

    assert_eq!(report.bifurcation_count(), 0);
    assert_eq!(report.branch_count(), 1); // one continuous branch across the sweep
}

#[test]
fn identical_inputs_yield_bit_identical_reports() {
    let (fields, states, parameter) = common::pitchfork();
    let first = continuation(&fields, &states, &parameter, &sweep_1d(), &config_1d()).unwrap();
    let second = continuation(&fields, &states, &parameter, &sweep_1d(), &config_1d()).unwrap();
    assert_eq!(first.to_canonical_string(), second.to_canonical_string());
}

#[test]
fn determinism_covers_the_hopf_case_too() {
    let (fields, states, parameter) = common::hopf();
    let sweep = Sweep::new(-1.0, 1.0, 21);
    let config = StabilityConfig::new(vec![(-1.5, 1.5), (-1.5, 1.5)]);
    let first = continuation(&fields, &states, &parameter, &sweep, &config).unwrap();
    let second = continuation(&fields, &states, &parameter, &sweep, &config).unwrap();
    assert_eq!(first.to_canonical_string(), second.to_canonical_string());
    // The localized Hopf parameter matches to the last bit.
    assert_eq!(
        first.bifurcations[0].parameter_value.to_bits(),
        second.bifurcations[0].parameter_value.to_bits()
    );
}

#[test]
fn samples_lie_exactly_on_the_sweep_grid() {
    let (fields, states, parameter) = common::saddle_node();
    let sweep = sweep_1d();
    let report = continuation(&fields, &states, &parameter, &sweep, &config_1d()).unwrap();
    let grid = sweep.grid();
    assert_eq!(report.samples.len(), grid.len());
    for (sample, &mu) in report.samples.iter().zip(&grid) {
        assert_eq!(sample.parameter_value.to_bits(), mu.to_bits());
    }
}

#[test]
fn empty_state_space_is_rejected() {
    let (fields, _states, parameter) = common::saddle_node();
    let error = continuation(&fields, &[], &parameter, &sweep_1d(), &config_1d()).unwrap_err();
    assert_eq!(error, lawsynth_bifurcation::BifurcationError::EmptyStateSpace);
}

#[test]
fn parameter_that_is_also_a_state_is_rejected() {
    let (fields, states, _parameter) = common::saddle_node();
    let x = Identifier::new("x").unwrap();
    let error = continuation(&fields, &states, &x, &sweep_1d(), &config_1d()).unwrap_err();
    assert_eq!(error, lawsynth_bifurcation::BifurcationError::ParameterIsState(x));
}

#[test]
fn an_ill_formed_sweep_is_rejected() {
    let (fields, states, parameter) = common::saddle_node();
    let bad = Sweep::new(1.0, -1.0, 21); // inverted range
    let error = continuation(&fields, &states, &parameter, &bad, &config_1d()).unwrap_err();
    assert!(matches!(error, lawsynth_bifurcation::BifurcationError::InvalidSweep(_)));
}

#[test]
fn a_stability_fault_is_reported_against_its_parameter_value() {
    // A search box of the wrong dimension makes every per-mu stability call fail;
    // the error must carry the offending parameter value, not be swallowed.
    let (fields, states, parameter) = common::saddle_node();
    let wrong = StabilityConfig::new(vec![(-2.0, 2.0), (-2.0, 2.0)]); // 2D box for a 1D system
    let error = continuation(&fields, &states, &parameter, &sweep_1d(), &wrong).unwrap_err();
    match error {
        lawsynth_bifurcation::BifurcationError::Stability { parameter_value, .. } => {
            assert_eq!(parameter_value.to_bits(), (-1.0_f64).to_bits());
        }
        other => panic!("expected a stability error, got {other:?}"),
    }
}
