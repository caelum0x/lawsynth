//! Recovery of the Michaelis-Menten rational law — the flagship case that
//! explicit `ẋ = Θ(x) ξ` regression cannot express.

mod support;

use lawsynth_implicit::{ImplicitConfig, implicit_discover};
use support::{coefficient, dataset_x, integrate, michaelis_menten};

#[test]
fn recovers_michaelis_menten_rational_law_degree_one() {
    let vmax = 1.5;
    let km = 0.3;
    let (time, xs) = integrate(michaelis_menten(vmax, km), 2.0, 0.01, 400);
    let dataset = dataset_x(time, xs);

    let config = ImplicitConfig { degree: 1, ..Default::default() };
    let result = implicit_discover(&dataset, &config).unwrap();

    // The implicit relation is the flagship deliverable and must be consistent.
    assert!(result.relation.consistent);
    assert!(result.relation.relative_residual < 1e-3);

    let law = result.rational_law.expect("rational law reconstructed");
    assert_eq!(law.target, "x");

    // Canonical form: Q = Km + x (monic in x), P = -Vmax·x.
    let recovered_km = coefficient(&law.denominator.terms, "1");
    let recovered_x_coeff = coefficient(&law.denominator.terms, "x");
    let recovered_vmax = -coefficient(&law.numerator.terms, "x");

    let km_error = (recovered_km - km).abs();
    let vmax_error = (recovered_vmax - vmax).abs();
    // Coefficient errors are reported so the honesty of the recovery is visible.
    assert!(km_error < 5e-3, "Km error {km_error} (recovered {recovered_km}, true {km})");
    assert!(vmax_error < 5e-3, "Vmax error {vmax_error} (recovered {recovered_vmax}, true {vmax})");
    assert!((recovered_x_coeff - 1.0).abs() < 1e-9, "denominator not monic in x");

    // The denominator Km + x stays positive across x ∈ (0, 2], so no pole.
    assert!(law.denominator_nonvanishing);
    assert!(law.min_abs_denominator > km - 1e-3);

    // The reconstructed law reproduces the true derivative at a probe point.
    let probe = 1.0;
    let expected = -vmax * probe / (km + probe);
    assert!((law.evaluate(&[probe]) - expected).abs() < 5e-3);
}

// Honest identifiability note: at degree 2 the augmented library
// Θ(x, ẋ) admits a MULTI-DIMENSIONAL nullspace — the true Michaelis-Menten
// relation `r(x,ẋ)=0` AND `x·r(x,ẋ)=0` both fit to machine precision — so
// implicit discovery is NOT guaranteed to return the minimal, spurious-free
// form at higher degree (a documented limit of implicit SR / SINDy-PI, stated in
// specs/implicit-dynamics/README.md). The achievable contract is that the
// discovered relation is still a VALID one that reproduces the true dynamics, and
// that the minimal library degree (1) recovers the clean rational law.
#[test]
fn degree_two_relation_is_valid_even_when_not_minimal() {
    let vmax = 1.0;
    let km = 0.6;
    let (time, xs) = integrate(michaelis_menten(vmax, km), 2.5, 0.01, 400);
    let dataset = dataset_x(time, xs);

    let config = ImplicitConfig { degree: 2, ..Default::default() };
    let result = implicit_discover(&dataset, &config).unwrap();

    // Whatever relation is selected, it fits to machine precision (a valid member
    // of the nullspace) and reconstructs an explicit rational law.
    assert!(result.relation.consistent);
    assert!(result.relation.relative_residual < 1e-6);
    let law = result.rational_law.expect("rational law reconstructed");

    // The reconstructed law reproduces the TRUE derivative across the window,
    // even if it carries a higher-degree (redundant) representation — this is the
    // honest guarantee, not spurious-term absence.
    for &probe in &[0.4_f64, 0.9, 1.4, 1.9] {
        let expected = -vmax * probe / (km + probe);
        assert!(
            (law.evaluate(&[probe]) - expected).abs() < 1e-2,
            "law.evaluate({probe}) = {} vs expected {expected}",
            law.evaluate(&[probe])
        );
    }
}

// The minimal (degree-1) library has a one-dimensional nullspace and recovers the
// clean Michaelis-Menten rational law with no spurious terms.
#[test]
fn degree_one_recovers_clean_rational_law() {
    let vmax = 1.0;
    let km = 0.6;
    let (time, xs) = integrate(michaelis_menten(vmax, km), 2.5, 0.01, 400);
    let dataset = dataset_x(time, xs);

    let config = ImplicitConfig { degree: 1, ..Default::default() };
    let result = implicit_discover(&dataset, &config).unwrap();
    let law = result.rational_law.expect("rational law reconstructed");

    let recovered_km = coefficient(&law.denominator.terms, "1");
    let recovered_vmax = -coefficient(&law.numerator.terms, "x");
    assert!((recovered_km - km).abs() < 2e-2, "Km recovered {recovered_km}");
    assert!((recovered_vmax - vmax).abs() < 2e-2, "Vmax recovered {recovered_vmax}");
}
