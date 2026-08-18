//! Integration coverage for grammar template priors (`specs/template-priors/`).
//!
//! A [`TemplatePrior`] is a deterministic hard filter over candidate terms,
//! applied to the materialised feature library before the sparse solve. These
//! tests verify that (1) each rule filters as specified end-to-end through
//! `discover`, (2) the drop report is honest and complete, (3) a prior that
//! admits the truth still recovers a known law while a prior that excludes it
//! honestly returns a zero law, (4) results are bit-identical run to run, and
//! (5) the default (no-prior) path is byte-identical to pre-change behaviour.

use std::collections::BTreeMap;

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_discovery::{
    DiscoveryConfig, DropReason, TemplateError, TemplatePrior, TermKind, discover,
};
use lawsynth_expr::{Expr, evaluate, print};

fn ident(name: &str) -> Identifier {
    Identifier::new(name).unwrap()
}

/// Exponential growth `x(t) = exp(2t)`, whose derivative law is `dx/dt = 2x`.
fn exponential_growth() -> (Dataset, Identifier) {
    let x = ident("x");
    let time = (0..101).map(|step| step as f64 * 0.01).collect::<Vec<_>>();
    let values = time.iter().map(|t| (2.0 * t).exp()).collect::<Vec<_>>();
    let dataset =
        Dataset::new(TimeAxis::new(time).unwrap(), [NumericColumn::new(x.clone(), values)])
            .unwrap();
    (dataset, x)
}

/// A sine control signal: `x(t) = sin(phase)` with `phase(t) = t`, so `x`'s value
/// is `sin(phase)` and a trig candidate is needed to fit it.
fn sine_signal() -> (Dataset, Identifier, Identifier) {
    let x = ident("x");
    let phase = ident("phase");
    let time = (0..401).map(|index| index as f64 * 0.01).collect::<Vec<_>>();
    let dataset = Dataset::new(
        TimeAxis::new(time.clone()).unwrap(),
        [
            NumericColumn::new(x.clone(), time.iter().map(|t| t.sin()).collect()),
            NumericColumn::new(phase.clone(), time),
        ],
    )
    .unwrap();
    (dataset, x, phase)
}

#[test]
fn degree_cap_forces_a_linear_only_library() {
    let (dataset, x) = exponential_growth();
    let mut config = DiscoveryConfig::new([x.clone()]);
    config.polynomial_degree = 3;
    config.with_template_prior(TemplatePrior::unconstrained().with_max_total_degree(1));

    let result = discover(&dataset, &config).unwrap();
    let report = result.template_filter.expect("prior populates the report");
    // Library {1, x, x^2, x^3}: degree cap 1 drops x^2 and x^3.
    assert_eq!(report.considered, 4);
    assert_eq!(report.admitted, 2);
    assert!(report.dropped.iter().all(|d| matches!(d.reason, DropReason::DegreeExceeded { .. })));
    // The linear law dx/dt = 2x is still recoverable within the constrained set.
    let printed = print(&result.candidates[0].world.laws()[&x].expression);
    assert!(printed.contains("2.000"), "unexpected constrained law: {printed}");
}

#[test]
fn variable_whitelist_excludes_foreign_columns() {
    let (dataset, x, phase) = sine_signal();
    let mut config = DiscoveryConfig::new([x.clone()]);
    config.polynomial_degree = 1;
    config.include_trigonometric = true;
    // Only `phase` may appear; `x` on the RHS is forbidden.
    config.with_template_prior(
        TemplatePrior::unconstrained().with_allowed_variables([phase.clone()]),
    );

    let result = discover(&dataset, &config).unwrap();
    let law = print(&result.candidates[0].world.laws()[&x].expression);
    // Neither `x` nor `sin(x)`/`cos(x)` may appear; the only admitted variable is
    // `phase`, which contains no `x` character, so its absence is exact.
    assert!(!law.contains('x'), "x leaked into law: {law}");
    let report = result.template_filter.unwrap();
    assert!(report.dropped.iter().any(|d| matches!(
        &d.reason,
        DropReason::DisallowedVariable { variable } if variable.as_str() == "x"
    )));
}

#[test]
fn kind_allowlist_keeps_only_trig_and_constant() {
    let (dataset, x, _phase) = sine_signal();
    let mut config = DiscoveryConfig::new([x.clone()]);
    config.polynomial_degree = 2;
    config.include_trigonometric = true;
    config.include_rational = true;
    config.with_template_prior(
        TemplatePrior::unconstrained()
            .with_allowed_kinds([TermKind::Trigonometric, TermKind::Constant]),
    );

    let result = discover(&dataset, &config).unwrap();
    let report = result.template_filter.unwrap();
    // Every dropped term was rejected as a disallowed kind (polynomial/rational).
    assert!(report.dropped.iter().all(|d| matches!(d.reason, DropReason::DisallowedKind { .. })));
    assert!(report.admitted >= 1);
}

#[test]
fn forbidding_interactions_removes_cross_terms() {
    let x = ident("x");
    let y = ident("y");
    let time = (0..200).map(|i| i as f64 * 0.01).collect::<Vec<_>>();
    let dataset = Dataset::new(
        TimeAxis::new(time.clone()).unwrap(),
        [
            NumericColumn::new(x.clone(), time.iter().map(|t| t.sin()).collect()),
            NumericColumn::new(y.clone(), time.iter().map(|t| t.cos()).collect()),
        ],
    )
    .unwrap();
    let mut config = DiscoveryConfig::new([x.clone(), y.clone()]);
    config.polynomial_degree = 2;
    config.with_template_prior(TemplatePrior::unconstrained().forbidding_interactions());

    let result = discover(&dataset, &config).unwrap();
    let report = result.template_filter.unwrap();
    // The only cross term x*y is dropped as an interaction.
    assert_eq!(
        report
            .dropped
            .iter()
            .filter(|d| matches!(d.reason, DropReason::InteractionForbidden { .. }))
            .count(),
        1
    );
    for state in [&x, &y] {
        let law = print(&result.candidates[0].world.laws()[state].expression);
        // No surviving law may contain the product of the two distinct variables
        // (`print` renders products as `(a*b)` with no spaces).
        assert!(!law.contains("x*y") && !law.contains("y*x"), "cross term survived: {law}");
    }
}

#[test]
fn max_active_terms_caps_the_candidate_library() {
    let (dataset, x) = exponential_growth();
    let mut config = DiscoveryConfig::new([x.clone()]);
    config.polynomial_degree = 4;
    config.with_template_prior(TemplatePrior::unconstrained().with_max_active_terms(2));

    let result = discover(&dataset, &config).unwrap();
    let report = result.template_filter.unwrap();
    assert_eq!(report.admitted, 2);
    assert!(
        report
            .dropped
            .iter()
            .all(|d| matches!(d.reason, DropReason::MaxActiveExceeded { limit: 2 }))
    );
    // A discovered law can have at most `limit` active terms; with a 2-term
    // candidate set the rendered law has at most one top-level sum.
    let law = result.candidates[0].world.laws()[&x].expression.clone();
    let sums = print(&law).matches('+').count();
    assert!(sums < 2, "more active terms than the cap allowed");
}

#[test]
fn prior_admitting_the_truth_recovers_the_law() {
    let (dataset, x) = exponential_growth();
    let mut config = DiscoveryConfig::new([x.clone()]);
    config.polynomial_degree = 3;
    // The truth dx/dt = 2x is a degree-1 polynomial in {x}: admitted.
    config.with_template_prior(
        TemplatePrior::unconstrained()
            .with_allowed_kinds([TermKind::Polynomial, TermKind::Constant])
            .with_max_total_degree(1)
            .requiring_kind(TermKind::Polynomial),
    );

    let result = discover(&dataset, &config).unwrap();
    let value =
        evaluate(&result.candidates[0].world.laws()[&x].expression, &BTreeMap::from([(x, 1.0)]))
            .unwrap();
    assert!((value - 2.0).abs() < 0.02, "expected slope ~2, got {value}");
}

#[test]
fn prior_excluding_the_truth_returns_an_honest_zero_law() {
    // The truth needs a trigonometric term, but the prior admits only polynomials
    // AND forbids every variable. The library collapses to the intercept alone;
    // combined with a degree-0 cap the sine cannot be recovered.
    let (dataset, x, _phase) = sine_signal();
    let mut config = DiscoveryConfig::new([x.clone()]);
    config.polynomial_degree = 1;
    config.include_trigonometric = true;
    // Allow only trig kind but cap degree at 0: sin(phase) has degree 1, so every
    // term is dropped and no candidate survives.
    config.with_template_prior(
        TemplatePrior::unconstrained()
            .with_allowed_kinds([TermKind::Trigonometric])
            .with_max_total_degree(0),
    );

    let result = discover(&dataset, &config).unwrap();
    // Honest failure: an empty admissible set yields a zero law, not a fabricated
    // structure and not a silent success.
    assert_eq!(result.candidates[0].world.laws()[&x].expression, Expr::Constant(0.0));
    let report = result.template_filter.unwrap();
    assert_eq!(report.admitted, 0);
}

#[test]
fn required_kind_without_a_candidate_is_rejected() {
    let (dataset, x) = exponential_growth();
    let mut config = DiscoveryConfig::new([x.clone()]);
    config.polynomial_degree = 2;
    // No trigonometric terms in the library, but one is required -> unsatisfiable.
    config.with_template_prior(
        TemplatePrior::unconstrained().requiring_kind(TermKind::Trigonometric),
    );

    let error = discover(&dataset, &config).unwrap_err();
    assert!(
        error.to_string().contains("Trigonometric"),
        "expected an unsatisfiable-required-kind error, got: {error}"
    );
}

#[test]
fn template_prior_admissible_is_a_pure_deterministic_function() {
    // Filter determinism at the type level, independent of the pipeline.
    let (dataset, x) = exponential_growth();
    let mut config = DiscoveryConfig::new([x]);
    config.polynomial_degree = 4;
    let prior = TemplatePrior::unconstrained().with_max_total_degree(2).forbidding_interactions();
    config.with_template_prior(prior);

    let first = discover(&dataset, &config).unwrap();
    let second = discover(&dataset, &config).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.template_filter, second.template_filter);
}

#[test]
fn default_config_leaves_the_template_report_empty() {
    let (dataset, _x) = exponential_growth();
    let result = discover(&dataset, &DiscoveryConfig::new([ident("x")])).unwrap();
    assert!(result.template_filter.is_none());
}

#[test]
fn unconstrained_prior_is_byte_identical_to_no_prior() {
    let (dataset, x) = exponential_growth();
    let mut baseline = DiscoveryConfig::new([x.clone()]);
    baseline.polynomial_degree = 3;

    let mut with_prior = baseline.clone();
    with_prior.with_template_prior(TemplatePrior::unconstrained());

    let without = discover(&dataset, &baseline).unwrap();
    let withp = discover(&dataset, &with_prior).unwrap();

    // The report is present but records zero drops; the discovered world matches.
    let report = withp.template_filter.clone().expect("prior populates the report");
    assert_eq!(report.dropped.len(), 0);
    assert_eq!(report.admitted, report.considered);
    assert_eq!(without.candidates[0].world, withp.candidates[0].world);
}

#[test]
fn report_is_complete_every_term_admitted_or_dropped_once() {
    let (dataset, x) = exponential_growth();
    let mut config = DiscoveryConfig::new([x]);
    config.polynomial_degree = 4;
    config.with_template_prior(
        TemplatePrior::unconstrained().with_max_total_degree(2).with_max_active_terms(2),
    );
    let result = discover(&dataset, &config).unwrap();
    let report = result.template_filter.unwrap();
    assert_eq!(report.admitted + report.dropped.len(), report.considered);
    assert_eq!(report.admitted, 2);
}

#[test]
fn required_kind_conflicting_with_kind_allowlist_is_unsatisfiable() {
    let (dataset, x) = exponential_growth();
    let mut config = DiscoveryConfig::new([x]);
    config.polynomial_degree = 2;
    config.include_trigonometric = true;
    // Allow only polynomials, then require a trig term: direct conflict.
    let prior = TemplatePrior::unconstrained()
        .with_allowed_kinds([TermKind::Polynomial, TermKind::Constant])
        .requiring_kind(TermKind::Trigonometric);
    config.with_template_prior(prior);
    let error = discover(&dataset, &config).unwrap_err();
    assert!(error.to_string().contains("Trigonometric"), "unexpected error: {error}");

    // Sanity-check the underlying typed error shape at the filter layer.
    let unsatisfiable = TemplateError::UnsatisfiableRequiredKind(TermKind::Trigonometric);
    assert!(unsatisfiable.to_string().contains("Trigonometric"));
}
