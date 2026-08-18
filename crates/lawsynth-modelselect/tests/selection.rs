//! Integration tests for the deterministic cross-validated model selection sweep.

mod common;

use std::collections::BTreeMap;

use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_discovery::{DiscoveryConfig, discover};
use lawsynth_expr::evaluate;
use lawsynth_modelselect::{
    CvConfig, CvScheme, FoldStatus, ModelSelectError, ScoreMetric, select_model,
    sweep_degrees_thresholds,
};

use common::{cubic_oscillator, id, linear_oscillator, logistic};

/// Greatest `mean_score` across the report's candidates, by total order.
fn max_mean_score(report: &lawsynth_modelselect::SelectionReport) -> f64 {
    report.candidates.iter().map(|candidate| candidate.mean_score).fold(
        f64::NEG_INFINITY,
        |acc, value| if value.total_cmp(&acc).is_gt() { value } else { acc },
    )
}

// --- Complexity selection ---------------------------------------------------

#[test]
fn selects_the_true_cubic_degree_and_beats_lower_degrees() {
    // True system is a Duffing cubic (degree 3). CV must select degree 3: the
    // degree-1 and degree-2 libraries cannot represent x^3, so they generalize
    // strictly worse on the held-out segments.
    let data = cubic_oscillator();
    let base = DiscoveryConfig::new([id("x"), id("v")]);
    let cv = CvConfig::new(3);
    let report = sweep_degrees_thresholds(&data, &base, &[1, 2, 3], &[0.05], &cv).unwrap();

    assert_eq!(report.best().config.polynomial_degree, 3, "cubic system selects degree 3");
    let degree1 = report.candidates[0].mean_score;
    let degree2 = report.candidates[1].mean_score;
    let degree3 = report.candidates[2].mean_score;
    assert!(degree3 > 0.99, "true degree generalizes almost perfectly: {degree3}");
    assert!(degree3 > degree1, "degree 3 beats underfitting degree 1: {degree3} vs {degree1}");
    assert!(degree3 > degree2, "degree 3 beats underfitting degree 2: {degree3} vs {degree2}");
}

#[test]
fn selected_cubic_model_recovers_the_law() {
    // Re-discover with the selected config on the full dataset and confirm the
    // recovered laws reproduce dx/dt = v and the cubic dv/dt = -0.3 v - x - x^3.
    let data = cubic_oscillator();
    let base = DiscoveryConfig::new([id("x"), id("v")]);
    let cv = CvConfig::new(3);
    let report = sweep_degrees_thresholds(&data, &base, &[1, 2, 3], &[0.05], &cv).unwrap();

    let mut best = base.clone();
    best.polynomial_degree = report.best().config.polynomial_degree;
    best.sparse.threshold = report.best().config.threshold;
    let discovered = discover(&data, &best).unwrap();
    let laws = discovered.candidates[0].world.laws();

    // dx/dt = v: at v = 1 the x-derivative is 1 regardless of x.
    let dx = evaluate(&laws[&id("x")].expression, &env(&[("x", 0.0), ("v", 1.0)])).unwrap();
    assert!((dx - 1.0).abs() < 0.05, "dx/dt should recover v, got {dx}");

    // dv/dt at (x=2, v=0) = -x - x^3 = -2 - 8 = -10; only a cubic library can
    // reach this while also matching -2 at x = 1.
    let dv = evaluate(&laws[&id("v")].expression, &env(&[("x", 2.0), ("v", 0.0)])).unwrap();
    assert!((dv + 10.0).abs() < 0.5, "dv/dt should recover the cubic, got {dv}");
}

#[test]
fn linear_system_selects_the_simplest_adequate_degree() {
    // A linear damped oscillator (degree 1). Higher-degree libraries prune their
    // spurious terms and generalize identically, so the documented simpler-model
    // tie-break must land on degree 1.
    let data = linear_oscillator();
    let base = DiscoveryConfig::new([id("x"), id("v")]);
    let cv = CvConfig::new(3);
    let report = sweep_degrees_thresholds(&data, &base, &[1, 2, 3], &[0.05], &cv).unwrap();

    assert_eq!(report.best().config.polynomial_degree, 1, "tie-break prefers the simpler degree");
    for candidate in &report.candidates {
        assert!(candidate.mean_score > 0.99, "every degree fits the linear law well");
    }
}

#[test]
fn logistic_system_selects_degree_two_over_underfit_and_overfit() {
    // Logistic (degree 2): degree 1 underfits the saturation, degree 3 prunes its
    // spurious x^3 back to the degree-2 model, so selection lands on degree 2.
    let data = logistic();
    let base = DiscoveryConfig::new([id("x")]);
    let cv = CvConfig::new(3);
    let report = sweep_degrees_thresholds(&data, &base, &[1, 2, 3], &[0.05], &cv).unwrap();

    assert_eq!(report.best().config.polynomial_degree, 2);
    let degree1 = report.candidates[0].mean_score;
    let degree2 = report.candidates[1].mean_score;
    let degree3 = report.candidates[2].mean_score;
    assert!(degree2 > degree1, "degree 2 beats underfitting degree 1");
    assert!(degree2 >= degree3, "degree 2 is at least as good as the more complex degree 3");
}

// --- Threshold selection ----------------------------------------------------

#[test]
fn threshold_that_prunes_real_terms_scores_worse() {
    // A too-large threshold prunes the genuine restoring/damping terms, leaving a
    // near-zero law that cannot predict the held-out oscillation.
    let data = linear_oscillator();
    let base = DiscoveryConfig::new([id("x"), id("v")]);
    let cv = CvConfig::new(3);
    let report = sweep_degrees_thresholds(&data, &base, &[2], &[0.05, 2.0], &cv).unwrap();

    let keep = report.candidates[0].mean_score; // threshold 0.05
    let prune = report.candidates[1].mean_score; // threshold 2.0
    assert!(keep > prune, "keeping real terms must generalize better: {keep} vs {prune}");
    assert_eq!(report.best().config.threshold, 0.05, "the smaller threshold is selected");
    assert!(prune < 0.0, "an over-pruned model predicts worse than the held-out mean");
}

// --- Report completeness & auditability -------------------------------------

#[test]
fn full_report_is_populated_and_best_points_to_the_max() {
    let data = cubic_oscillator();
    let base = DiscoveryConfig::new([id("x"), id("v")]);
    let cv = CvConfig::new(3);
    let report = sweep_degrees_thresholds(&data, &base, &[1, 2, 3], &[0.05], &cv).unwrap();

    assert_eq!(report.candidates.len(), 3);
    assert_eq!(report.folds, 3);
    for (index, candidate) in report.candidates.iter().enumerate() {
        assert_eq!(candidate.grid_index, index);
        assert_eq!(candidate.fold_scores.len(), 3, "every candidate has per-fold scores");
        for (fold_index, fold) in candidate.fold_scores.iter().enumerate() {
            assert_eq!(fold.fold_index, fold_index);
            // Forward-chaining: test range immediately follows the training data.
            assert_eq!(fold.train_range.0, 0, "forward chaining trains from the start");
            assert_eq!(fold.test_range.0, fold.train_range.1, "test follows train");
            assert!(fold.test_range.1 > fold.test_range.0);
        }
    }
    // best_index truly points at the maximum mean score.
    assert!((report.best().mean_score - max_mean_score(&report)).abs() < f64::EPSILON);

    // The rendered audit table lists every candidate and marks the winner.
    let table = report.render_table();
    assert!(table.contains("<=="));
    assert_eq!(table.matches("grid[").count(), 3);
}

// --- Determinism ------------------------------------------------------------

#[test]
fn selection_is_bit_identical_across_runs() {
    let data = cubic_oscillator();
    let base = DiscoveryConfig::new([id("x"), id("v")]);
    let cv = CvConfig::new(3);
    let first = sweep_degrees_thresholds(&data, &base, &[1, 2, 3], &[0.05, 0.1], &cv).unwrap();
    let second = sweep_degrees_thresholds(&data, &base, &[1, 2, 3], &[0.05, 0.1], &cv).unwrap();

    assert_eq!(first.best_index, second.best_index);
    assert_eq!(first.candidates.len(), second.candidates.len());
    for (a, b) in first.candidates.iter().zip(&second.candidates) {
        assert_eq!(a.mean_score.to_bits(), b.mean_score.to_bits(), "mean score bits");
        assert_eq!(a.active_terms, b.active_terms);
        assert_eq!(a.failed_folds, b.failed_folds);
        assert_eq!(a.fold_scores.len(), b.fold_scores.len());
        for (fa, fb) in a.fold_scores.iter().zip(&b.fold_scores) {
            assert_eq!(fa.status, fb.status);
            assert_eq!(fa.score.to_bits(), fb.score.to_bits(), "fold score bits");
            assert_eq!(
                fa.r_squared.map(f64::to_bits),
                fb.r_squared.map(f64::to_bits),
                "fold r^2 bits"
            );
            assert_eq!(fa.rmse.map(f64::to_bits), fb.rmse.map(f64::to_bits), "fold rmse bits");
        }
    }
}

// --- Alternate scheme and metric --------------------------------------------

#[test]
fn rolling_blocks_scheme_also_selects_the_cubic_degree() {
    let data = cubic_oscillator();
    let base = DiscoveryConfig::new([id("x"), id("v")]);
    let cv = CvConfig::new(3).with_scheme(CvScheme::RollingBlocks);
    let report = sweep_degrees_thresholds(&data, &base, &[1, 2, 3], &[0.05], &cv).unwrap();
    assert_eq!(report.best().config.polynomial_degree, 3);
    // Rolling blocks train on a single fixed-width block, not from the start.
    let first_fold = &report.candidates[0].fold_scores[1];
    assert!(first_fold.train_range.0 > 0, "rolling blocks slide the training window");
}

#[test]
fn rmse_metric_also_selects_the_cubic_degree() {
    let data = cubic_oscillator();
    let base = DiscoveryConfig::new([id("x"), id("v")]);
    let cv = CvConfig::new(3).with_metric(ScoreMetric::Rmse);
    let report = sweep_degrees_thresholds(&data, &base, &[1, 2, 3], &[0.05], &cv).unwrap();
    assert_eq!(report.best().config.polynomial_degree, 3);
    // Under the RMSE metric every fold score is a negated RMSE (<= 0).
    for candidate in &report.candidates {
        for fold in &candidate.fold_scores {
            assert!(fold.score <= 0.0);
        }
    }
}

#[test]
fn convenience_sweep_builds_the_full_degree_threshold_grid() {
    let data = linear_oscillator();
    let base = DiscoveryConfig::new([id("x"), id("v")]);
    let cv = CvConfig::new(3);
    let report = sweep_degrees_thresholds(&data, &base, &[1, 2], &[0.05, 0.1, 0.2], &cv).unwrap();
    assert_eq!(report.candidates.len(), 6, "2 degrees x 3 thresholds");
    // Grid order is degrees outer, thresholds inner.
    assert_eq!(report.candidates[0].config.polynomial_degree, 1);
    assert_eq!(report.candidates[0].config.threshold, 0.05);
    assert_eq!(report.candidates[3].config.polynomial_degree, 2);
    assert_eq!(report.candidates[3].config.threshold, 0.05);
}

// --- Error paths ------------------------------------------------------------

#[test]
fn empty_grid_is_rejected() {
    let data = linear_oscillator();
    let cv = CvConfig::new(3);
    assert_eq!(select_model(&data, &[], &cv), Err(ModelSelectError::EmptyGrid));
}

#[test]
fn zero_folds_is_rejected() {
    let data = linear_oscillator();
    let base = DiscoveryConfig::new([id("x"), id("v")]);
    let cv = CvConfig::new(0);
    assert_eq!(select_model(&data, &[base], &cv), Err(ModelSelectError::InvalidFoldCount));
}

#[test]
fn dataset_too_short_to_split_is_rejected() {
    // Six samples cannot form five folds with three train and two test each.
    let time = (0..6).map(|k| k as f64).collect::<Vec<_>>();
    let values = time.iter().map(|t| t.sin()).collect::<Vec<_>>();
    let data =
        Dataset::new(TimeAxis::new(time).unwrap(), [NumericColumn::new(id("x"), values)]).unwrap();
    let base = DiscoveryConfig::new([id("x")]);
    let cv = CvConfig::new(5);
    assert!(matches!(
        select_model(&data, &[base], &cv),
        Err(ModelSelectError::DatasetTooShort { samples: 6, folds: 5, .. })
    ));
}

#[test]
fn a_candidate_that_fails_discovery_is_recorded_as_fold_failures() {
    // The broken candidate caps the feature library below the degree-2 expansion,
    // so discovery errors on every fold — and on the full-data refit. It must be
    // recorded as per-fold failures, kept in the report, and never selected.
    let data = cubic_oscillator();
    let good = DiscoveryConfig::new([id("x"), id("v")]);
    let mut broken = DiscoveryConfig::new([id("x"), id("v")]);
    broken.polynomial_degree = 2;
    broken.resource_limits.max_features = 1;

    let cv = CvConfig::new(3);
    let report = select_model(&data, &[good, broken], &cv).unwrap();

    let broken_candidate = &report.candidates[1];
    assert_eq!(broken_candidate.failed_folds, 3, "all folds fail for the broken candidate");
    for fold in &broken_candidate.fold_scores {
        assert_eq!(fold.status, FoldStatus::DiscoveryFailed);
        assert!(fold.r_squared.is_none(), "a failed fold reports no real score");
    }
    assert!(broken_candidate.active_terms.is_none(), "full-data refit also fails");
    assert_eq!(report.best_index, 0, "the healthy candidate is selected");
    assert!(report.best().mean_score > broken_candidate.mean_score);
}

/// Builds an evaluation environment from `(name, value)` pairs.
fn env(pairs: &[(&str, f64)]) -> BTreeMap<lawsynth_core::Identifier, f64> {
    pairs.iter().map(|(name, value)| (id(name), *value)).collect()
}
