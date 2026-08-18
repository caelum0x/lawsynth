//! Integration tests for conserved-quantity detection.
//!
//! Each test builds a discovered vector field as `Expr` trees and asserts the
//! recovered invariants (or their honest absence) against the known physics.

use lawsynth_core::Identifier;
use lawsynth_expr::{Expr, UnaryOperator};
use lawsynth_invariants::{InvariantConfig, InvariantError, detect_invariants};

fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

fn symbol(name: &str) -> Expr {
    Expr::symbol(id(name))
}

fn negate(expr: Expr) -> Expr {
    Expr::unary(UnaryOperator::Negate, expr)
}

fn scaled(factor: f64, name: &str) -> Expr {
    Expr::product(Expr::constant(factor), symbol(name))
}

/// `ẋ = y, ẏ = -x` — the harmonic oscillator, energy `x² + y²`.
fn harmonic_oscillator() -> (Vec<(Identifier, Expr)>, Vec<Identifier>) {
    let fields = vec![(id("x"), symbol("y")), (id("y"), negate(symbol("x")))];
    (fields, vec![id("x"), id("y")])
}

#[test]
fn recovers_harmonic_oscillator_energy() {
    let (fields, states) = harmonic_oscillator();
    let config = InvariantConfig::default();
    let report = detect_invariants(&fields, &states, &config).unwrap();

    assert_eq!(report.invariants.len(), 1, "exactly one conserved quantity");
    let invariant = &report.invariants[0];
    let labels = &report.basis_labels;

    let x2 = invariant.coefficient(labels, "x^2").unwrap();
    let y2 = invariant.coefficient(labels, "y^2").unwrap();
    // Energy: equal, nonzero weight on the two squares.
    assert!(x2.abs() > 0.5, "x^2 weight should be substantial, got {x2}");
    assert!((x2 - y2).abs() < 1e-9, "x^2 and y^2 weights should match: {x2} vs {y2}");

    // Every other basis direction is negligible.
    for label in ["x", "y", "x*y"] {
        let value = invariant.coefficient(labels, label).unwrap();
        assert!(value.abs() < 1e-9, "spurious weight {value} on {label}");
    }
    assert!(invariant.residual < 1e-9, "residual {} too large", invariant.residual);
    assert!(invariant.singular_value < 1e-9);
}

#[test]
fn recovered_energy_is_canonically_normalized() {
    let (fields, states) = harmonic_oscillator();
    let report = detect_invariants(&fields, &states, &InvariantConfig::default()).unwrap();
    let coefficients = &report.invariants[0].coefficients;

    // Unit Euclidean norm.
    let norm: f64 = coefficients.iter().map(|c| c * c).sum::<f64>().sqrt();
    assert!((norm - 1.0).abs() < 1e-12, "coefficients should have unit norm, got {norm}");

    // Sign convention: the largest-magnitude entry is positive.
    let pivot = coefficients
        .iter()
        .cloned()
        .fold(0.0_f64, |acc, c| if c.abs() > acc.abs() { c } else { acc });
    assert!(pivot > 0.0, "largest-magnitude coefficient should be positive");
}

#[test]
fn recovers_undamped_pendulum_energy_with_trigonometric_library() {
    // ẋ = y, ẏ = -sin x. Energy ½y² + (1 - cos x); the invariant is ∝ y² - 2·cos x.
    let fields = vec![
        (id("x"), symbol("y")),
        (id("y"), negate(Expr::unary(UnaryOperator::Sin, symbol("x")))),
    ];
    let states = vec![id("x"), id("y")];
    let config = InvariantConfig {
        degree: 2,
        include_trigonometric: true,
        sample_lo: -1.2,
        sample_hi: 1.4,
        resolution: 5,
        tolerance: 1e-9,
    };
    let report = detect_invariants(&fields, &states, &config).unwrap();

    assert_eq!(report.invariants.len(), 1, "one conserved quantity for the pendulum");
    let invariant = &report.invariants[0];
    let labels = &report.basis_labels;

    let y2 = invariant.coefficient(labels, "y^2").unwrap();
    let cos_x = invariant.coefficient(labels, "cos(x)").unwrap();
    // H ∝ y² - 2·cos x, so cos(x) weight is -2× the y² weight.
    assert!(y2.abs() > 0.1, "y^2 weight should be substantial, got {y2}");
    assert!((cos_x + 2.0 * y2).abs() < 1e-8, "expected cos(x) = -2·y²: {cos_x} vs {y2}");

    // No other library term participates.
    for label in ["x", "y", "x^2", "x*y", "sin(x)", "sin(y)", "cos(y)"] {
        let value = invariant.coefficient(labels, label).unwrap();
        assert!(value.abs() < 1e-8, "spurious weight {value} on {label}");
    }
    assert!(invariant.residual < 1e-8, "residual {} too large", invariant.residual);
}

#[test]
fn polynomial_library_finds_no_spurious_pendulum_invariant() {
    // Honest limitation: the pendulum energy is transcendental (needs cos x).
    // A purely polynomial library MUST NOT invent a spurious invariant.
    let fields = vec![
        (id("x"), symbol("y")),
        (id("y"), negate(Expr::unary(UnaryOperator::Sin, symbol("x")))),
    ];
    let states = vec![id("x"), id("y")];
    let config = InvariantConfig {
        degree: 3,
        include_trigonometric: false,
        sample_lo: -1.2,
        sample_hi: 1.4,
        resolution: 6,
        tolerance: 1e-9,
    };
    let report = detect_invariants(&fields, &states, &config).unwrap();
    assert!(
        report.invariants.is_empty(),
        "a polynomial library cannot express the pendulum energy: {:?}",
        report.invariants
    );
}

#[test]
fn damped_oscillator_has_no_conserved_quantity() {
    // ẋ = y, ẏ = -x - 0.3y. Damping destroys conservation → empty report.
    let fields = vec![
        (id("x"), symbol("y")),
        (id("y"), Expr::difference(negate(symbol("x")), scaled(0.3, "y"))),
    ];
    let states = vec![id("x"), id("y")];
    let report = detect_invariants(&fields, &states, &InvariantConfig::default()).unwrap();
    assert!(
        report.invariants.is_empty(),
        "damped oscillator must yield no invariant, got {:?}",
        report.invariants
    );
}

#[test]
fn recovers_two_independent_invariants_from_a_decoupled_system() {
    // Two uncoupled oscillators with distinct frequencies:
    //   ẋ = y, ẏ = -x            (energy x² + y²)
    //   ż = w, ẇ = -4z           (energy 4z² + w²)
    // Distinct frequencies avoid the isotropic degeneracy that would create
    // extra cross-invariants, so the conserved space is exactly 2-dimensional.
    let fields = vec![
        (id("x"), symbol("y")),
        (id("y"), negate(symbol("x"))),
        (id("z"), symbol("w")),
        (id("w"), scaled(-4.0, "z")),
    ];
    let states = vec![id("x"), id("y"), id("z"), id("w")];
    let config = InvariantConfig {
        degree: 2,
        include_trigonometric: false,
        sample_lo: -1.0,
        sample_hi: 1.3,
        resolution: 3,
        tolerance: 1e-9,
    };
    let report = detect_invariants(&fields, &states, &config).unwrap();
    assert_eq!(report.invariants.len(), 2, "two independent conserved quantities");
    let labels = &report.basis_labels;

    // Every returned invariant lies in span{ x²+y², 4z²+w² }: it may only load
    // the four squares, with x²==y² and z²==4·w², and nothing else.
    for invariant in &report.invariants {
        let x2 = invariant.coefficient(labels, "x^2").unwrap();
        let y2 = invariant.coefficient(labels, "y^2").unwrap();
        let z2 = invariant.coefficient(labels, "z^2").unwrap();
        let w2 = invariant.coefficient(labels, "w^2").unwrap();
        assert!((x2 - y2).abs() < 1e-8, "x²/y² weights must match: {x2} vs {y2}");
        assert!((z2 - 4.0 * w2).abs() < 1e-8, "z² must be 4×w²: {z2} vs {w2}");
        assert!(invariant.residual < 1e-8, "residual {} too large", invariant.residual);

        for label in ["x", "y", "z", "w", "x*y", "x*z", "x*w", "y*z", "y*w", "z*w"] {
            let value = invariant.coefficient(labels, label).unwrap();
            assert!(value.abs() < 1e-8, "spurious weight {value} on {label}");
        }
    }

    // The two invariants are genuinely independent: one favours the (x,y)
    // subsystem, the other the (z,w) subsystem.
    let first_xy = report.invariants[0].coefficient(labels, "x^2").unwrap().abs();
    let second_xy = report.invariants[1].coefficient(labels, "x^2").unwrap().abs();
    assert!(
        (first_xy - second_xy).abs() > 1e-6,
        "the two invariants should not be identical projections"
    );
}

#[test]
fn identical_inputs_produce_bit_identical_reports() {
    let (fields, states) = harmonic_oscillator();
    let config = InvariantConfig::default();
    let first = detect_invariants(&fields, &states, &config).unwrap();
    let second = detect_invariants(&fields, &states, &config).unwrap();
    assert_eq!(first.to_bits(), second.to_bits(), "reports must be bit-identical");
    assert_eq!(first.basis_labels, second.basis_labels);
}

#[test]
fn rejects_empty_states() {
    let fields = vec![(id("x"), symbol("x"))];
    let error = detect_invariants(&fields, &[], &InvariantConfig::default()).unwrap_err();
    assert_eq!(error, InvariantError::NoStates);
}

#[test]
fn rejects_empty_fields() {
    let error = detect_invariants(&[], &[id("x")], &InvariantConfig::default()).unwrap_err();
    assert_eq!(error, InvariantError::EmptyFields);
}

#[test]
fn rejects_a_state_without_a_field() {
    let fields = vec![(id("x"), symbol("y"))];
    let states = vec![id("x"), id("y")];
    let error = detect_invariants(&fields, &states, &InvariantConfig::default()).unwrap_err();
    assert_eq!(error, InvariantError::MissingField(id("y")));
}

#[test]
fn rejects_a_duplicated_state() {
    let fields = vec![(id("x"), symbol("x"))];
    let states = vec![id("x"), id("x")];
    let error = detect_invariants(&fields, &states, &InvariantConfig::default()).unwrap_err();
    assert_eq!(error, InvariantError::DuplicateState(id("x")));
}

#[test]
fn rejects_a_field_with_an_unknown_symbol() {
    // The field references parameter `k`, which is not a declared state.
    let fields =
        vec![(id("x"), symbol("y")), (id("y"), Expr::product(negate(symbol("k")), symbol("x")))];
    let states = vec![id("x"), id("y")];
    let error = detect_invariants(&fields, &states, &InvariantConfig::default()).unwrap_err();
    assert_eq!(error, InvariantError::UnknownSymbol(id("k")));
}

#[test]
fn rejects_invalid_configuration() {
    let (fields, states) = harmonic_oscillator();
    let zero_degree = InvariantConfig { degree: 0, ..InvariantConfig::default() };
    assert_eq!(
        detect_invariants(&fields, &states, &zero_degree).unwrap_err(),
        InvariantError::InvalidDegree
    );
    let coarse = InvariantConfig { resolution: 1, ..InvariantConfig::default() };
    assert_eq!(
        detect_invariants(&fields, &states, &coarse).unwrap_err(),
        InvariantError::InvalidResolution
    );
    let bad_box = InvariantConfig { sample_lo: 2.0, sample_hi: 1.0, ..InvariantConfig::default() };
    assert_eq!(
        detect_invariants(&fields, &states, &bad_box).unwrap_err(),
        InvariantError::InvalidBox
    );
    let bad_tolerance = InvariantConfig { tolerance: -1.0, ..InvariantConfig::default() };
    assert_eq!(
        detect_invariants(&fields, &states, &bad_tolerance).unwrap_err(),
        InvariantError::InvalidTolerance
    );
}

#[test]
fn detects_a_higher_degree_polynomial_invariant() {
    // A conservative nonlinear (Duffing-type) oscillator: ẋ = y, ẏ = -x - x³.
    // Energy H = ½y² + ½x² + ¼x⁴ is conserved; a degree-4 library recovers it.
    let cubic = Expr::product(Expr::product(symbol("x"), symbol("x")), symbol("x"));
    let fields =
        vec![(id("x"), symbol("y")), (id("y"), Expr::difference(negate(symbol("x")), cubic))];
    let states = vec![id("x"), id("y")];
    let config = InvariantConfig {
        degree: 4,
        include_trigonometric: false,
        sample_lo: -1.1,
        sample_hi: 1.3,
        resolution: 6,
        tolerance: 1e-9,
    };
    let report = detect_invariants(&fields, &states, &config).unwrap();
    assert_eq!(report.invariants.len(), 1, "one energy invariant for the Duffing oscillator");
    let invariant = &report.invariants[0];
    let labels = &report.basis_labels;

    let x2 = invariant.coefficient(labels, "x^2").unwrap();
    let y2 = invariant.coefficient(labels, "y^2").unwrap();
    let x4 = invariant.coefficient(labels, "x^4").unwrap();
    // H ∝ 2x² + 2y² + x⁴ (scaling ½y²+½x²+¼x⁴ by 4): x² == y², x⁴ == ½·x².
    assert!((x2 - y2).abs() < 1e-8, "x² and y² weights should match: {x2} vs {y2}");
    assert!((x4 - 0.5 * x2).abs() < 1e-8, "x⁴ should be half of x²: {x4} vs {x2}");
    assert!(invariant.residual < 1e-8, "residual {} too large", invariant.residual);
}
