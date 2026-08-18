//! Per-preset shape: feature config, template prior, and unit hints.

use lawsynth_core::Identifier;
use lawsynth_discovery::{DropReason, TermKind};
use lawsynth_domains::preset;
use lawsynth_features::FeatureLibrary;
use lawsynth_units::Dimension;

fn ident(name: &str) -> Identifier {
    Identifier::new(name).unwrap()
}

#[test]
fn oscillator_shape_is_linear_with_si_units_and_no_prior() {
    let osc = preset("damped-oscillator").unwrap();
    let config = osc.feature_config();
    assert_eq!(config.polynomial_degree, 1);
    assert!(config.include_constant);
    assert!(!osc.include_trigonometric());
    assert!(!osc.include_rational());
    assert!(osc.template_prior().is_none());

    // Honest SI unit hints: position in metres, velocity in metres per second.
    let hints = osc.unit_hints();
    assert_eq!(hints.len(), 2);
    let names: Vec<&str> = hints.iter().map(|hint| hint.variable.as_str()).collect();
    assert!(names.contains(&"x") && names.contains(&"v"));
    // Both hints carry a genuine (non-dimensionless) physical dimension.
    assert!(hints.iter().all(|hint| hint.unit.dimension() != Dimension::DIMENSIONLESS));

    // The discovery config mirrors the feature config.
    let discovery = osc.discovery_config();
    assert_eq!(discovery.polynomial_degree, 1);
    assert!(discovery.template_prior.is_none());
}

#[test]
fn lotka_prior_drops_only_the_constant_intercept() {
    let lotka = preset("lotka-volterra").unwrap();
    assert_eq!(lotka.feature_config().polynomial_degree, 2);
    assert!(lotka.unit_hints().is_empty(), "abstract populations carry no unit hint");

    let prior = lotka.template_prior().expect("lotka-volterra ships a structural prior");

    // Apply the prior to the same degree-2 library discovery materialises. The
    // ecological prior (polynomial kinds only) drops exactly the constant "1".
    let library = FeatureLibrary::polynomial([ident("predator"), ident("prey")], 2, true).unwrap();
    let selection = prior.admissible(library.terms()).unwrap();
    assert_eq!(selection.report.dropped_count(), 1);
    // The single drop is the constant intercept, rejected as a disallowed kind.
    assert_eq!(
        selection.report.dropped[0].reason,
        DropReason::DisallowedKind { kind: TermKind::Constant }
    );
    // The interaction and quadratic terms the true law (may) need survive.
    let admitted: Vec<&str> =
        selection.admitted.iter().map(|&i| library.terms()[i].name.as_str()).collect();
    assert!(admitted.iter().all(|name| *name != "1"));
    assert_eq!(admitted.len(), library.terms().len() - 1);
}

#[test]
fn brusselator_shape_is_cubic_with_no_prior() {
    let brusselator = preset("brusselator").unwrap();
    assert_eq!(brusselator.feature_config().polynomial_degree, 3);
    assert!(!brusselator.include_trigonometric());
    assert!(!brusselator.include_rational());
    // The constant source term forbids a kind/degree prior, so none is attached.
    assert!(brusselator.template_prior().is_none());
    assert!(brusselator.unit_hints().is_empty());
}

#[test]
fn state_variables_match_reference_variables() {
    for preset in lawsynth_domains::all() {
        assert_eq!(preset.state_variables(), preset.reference().variables());
        assert!(!preset.state_variables().is_empty());
    }
}
