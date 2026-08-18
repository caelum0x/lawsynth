//! End-to-end cross-crate pipeline tests.
//!
//! Each test drives the REAL pipeline across several crates on data whose true
//! law is known, and asserts a meaningful property — not merely that nothing
//! panicked. The key value is cross-crate *consistency*: independent crates must
//! agree about the same system (a stable field has negative Lyapunov exponents;
//! simplification preserves dynamics; a conservative field is a center with a
//! conserved energy and zero exponents).
//!
//! Discovery from finite RK4 data is not machine-exact, so tolerances are chosen
//! for that reality and documented at each assertion.

mod common;

use lawsynth_core::Identifier;
use lawsynth_egraph::{RewriteConfig, simplify_law};
use lawsynth_expr::{Environment, Expr, evaluate, print};
use lawsynth_invariants::{InvariantConfig, detect_invariants};
use lawsynth_lyapunov::{LyapunovConfig, lyapunov_spectrum};
use lawsynth_stability::{Classification, StabilityConfig, analyze_stability};

use lawsynth_discovery::{DiscoveryConfig, discover};

/// Discovers the first candidate world for a dataset and returns its field
/// `(state, expression)` pairs plus the state ordering — the exact conversion
/// the CLI's analysis commands perform on a discovered world.
fn discover_fields(dataset: &lawsynth_data::Dataset) -> (Vec<(Identifier, Expr)>, Vec<Identifier>) {
    let config = DiscoveryConfig::new([common::id("x"), common::id("y")]);
    let result = discover(dataset, &config).expect("discovery succeeds on clean data");
    let world = &result.candidates[0].world;
    let states: Vec<Identifier> = world.state_ids().cloned().collect();
    let fields: Vec<(Identifier, Expr)> =
        world.laws().iter().map(|(target, law)| (target.clone(), law.expression.clone())).collect();
    (fields, states)
}

#[test]
fn discover_then_stability_finds_a_stable_origin() {
    // discovery -> world -> stability. The damped oscillator's origin is a stable
    // spiral; discovery + linear-stability must agree on that.
    let (fields, states) = discover_fields(&common::damped_oscillator_dataset());
    let config = StabilityConfig::new(vec![(-2.0, 2.0), (-2.0, 2.0)]);
    let report = analyze_stability(&fields, &states, &config).unwrap();

    let origin = report
        .fixed_points
        .iter()
        .find(|point| point.coordinates.iter().all(|c| c.abs() < 1e-2))
        .expect("a fixed point at the origin");
    assert_eq!(origin.classification, Classification::StableSpiral);
}

#[test]
fn discover_then_lyapunov_is_dissipative() {
    // discovery -> world -> lyapunov. A linear dissipative field has both
    // exponents negative and their sum equal to the trace (-0.3).
    let (fields, states) = discover_fields(&common::damped_oscillator_dataset());
    let config = LyapunovConfig::default().with_steps(20000);
    let report = lyapunov_spectrum(&fields, &states, &[1.0, 0.0], &config).unwrap();

    assert!(report.exponents().iter().all(|&l| l < 1e-3), "all exponents negative");
    // The sum (mean divergence) is the tight quantity; the trace is exactly -0.3.
    assert!((report.sum() - (-0.3)).abs() < 5e-2, "sum {} ~ trace -0.3", report.sum());
}

#[test]
fn stability_and_lyapunov_agree_on_dissipation() {
    // Cross-crate consistency: "stable" (stability) <=> "negative largest
    // exponent" (lyapunov) for the same discovered field.
    let (fields, states) = discover_fields(&common::damped_oscillator_dataset());

    let stability =
        analyze_stability(&fields, &states, &StabilityConfig::new(vec![(-2.0, 2.0), (-2.0, 2.0)]))
            .unwrap();
    let is_stable = stability.fixed_points.iter().any(|p| {
        matches!(p.classification, Classification::StableSpiral | Classification::StableNode)
    });

    let lyap = lyapunov_spectrum(
        &fields,
        &states,
        &[1.0, 0.0],
        &LyapunovConfig::default().with_steps(20000),
    )
    .unwrap();
    let negative_largest = lyap.largest() < 0.0;

    assert_eq!(is_stable, negative_largest, "stability and lyapunov must agree");
}

#[test]
fn conservative_system_is_a_center_with_energy_and_zero_exponents() {
    // Three crates agreeing on "conservative": stability -> center (inconclusive),
    // invariants -> energy x^2 + y^2, lyapunov -> exponents ~ 0.
    let (fields, states) = discover_fields(&common::harmonic_oscillator_dataset());

    let stability =
        analyze_stability(&fields, &states, &StabilityConfig::new(vec![(-2.0, 2.0), (-2.0, 2.0)]))
            .unwrap();
    let origin = stability
        .fixed_points
        .iter()
        .find(|p| p.coordinates.iter().all(|c| c.abs() < 1e-2))
        .expect("origin fixed point");
    assert_eq!(origin.classification, Classification::Center);

    let invariants = detect_invariants(&fields, &states, &InvariantConfig::default()).unwrap();
    assert_eq!(invariants.invariants.len(), 1, "one conserved quantity");
    let energy = &invariants.invariants[0];
    let x2 = energy.coefficient(&invariants.basis_labels, "x^2").unwrap();
    let y2 = energy.coefficient(&invariants.basis_labels, "y^2").unwrap();
    assert!((x2 - y2).abs() < 1e-6, "energy is x^2 + y^2 (equal weights)");
    assert!(energy.residual < 1e-6);

    let lyap = lyapunov_spectrum(
        &fields,
        &states,
        &[1.0, 0.0],
        &LyapunovConfig::default().with_steps(20000),
    )
    .unwrap();
    assert!(lyap.exponents().iter().all(|&l| l.abs() < 1e-2), "conservative => zero exponents");
}

#[test]
fn simplify_preserves_dynamics() {
    // discovery -> egraph simplify -> the simplified fields must be numerically
    // identical to the originals wherever both are defined.
    let (fields, _states) = discover_fields(&common::damped_oscillator_dataset());
    let simplified = simplify_law(&fields, &RewriteConfig::default()).unwrap();
    assert_eq!(fields.len(), simplified.len());

    // Sample a small deterministic lattice of (x, y) points and compare.
    for xi in [-1.0, -0.3, 0.0, 0.7, 1.4] {
        for yi in [-1.1, 0.0, 0.5, 1.2] {
            let env = Environment::from([(common::id("x"), xi), (common::id("y"), yi)]);
            for ((_, original), (_, simple)) in fields.iter().zip(&simplified) {
                let a = evaluate(original, &env).unwrap();
                let b = evaluate(simple, &env).unwrap();
                assert!((a - b).abs() < 1e-9, "simplify changed the value at ({xi},{yi})");
            }
        }
    }
}

#[test]
fn pipeline_is_deterministic() {
    // The whole discover->render chain is bit-reproducible: identical rendered
    // laws across two independent runs on the same fixture.
    let dataset = common::damped_oscillator_dataset();
    let (fields_a, _) = discover_fields(&dataset);
    let (fields_b, _) = discover_fields(&dataset);
    let render = |fields: &[(Identifier, Expr)]| -> Vec<String> {
        fields.iter().map(|(t, e)| format!("{}={}", t.as_str(), print(e))).collect()
    };
    assert_eq!(render(&fields_a), render(&fields_b));
}
