//! The primary correctness guard for the rewrite engine.
//!
//! For a battery of expressions we compare the value of the *original* against
//! the value of its *simplified* form at many pseudo-random sample points. The
//! contract enforced is value preservation on the original's domain: wherever
//! the original evaluates to a finite number, the simplified form MUST evaluate
//! to the same number (to a tight tolerance). Points where the original is
//! undefined (division by zero, `log` of a non-positive value, overflow) are
//! skipped, since a documented rewrite may legitimately extend the domain.
//!
//! Any unsound rewrite — one that changes the function's value where it is
//! defined — makes this test fail.

use lawsynth_core::Identifier;
use lawsynth_egraph::{RewriteConfig, expression_cost, simplify_expr};
use lawsynth_expr::{Environment, evaluate, parse, symbols};

/// Samples drawn per expression.
const SAMPLES_PER_EXPRESSION: usize = 256;
/// Sampling half-width: symbols range over `[-SAMPLE_RANGE, SAMPLE_RANGE]`.
const SAMPLE_RANGE: f64 = 3.5;
/// Value-agreement tolerance (mixed absolute / relative).
const TOLERANCE: f64 = 1e-12;

/// A deterministic linear-congruential generator so the whole battery is
/// reproducible with no external crates.
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_unit(&mut self) -> f64 {
        // Numerical Recipes LCG constants.
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        // Top 53 bits give a uniform value in [0, 1).
        (self.state >> 11) as f64 / (1u64 << 53) as f64
    }

    fn next_in_range(&mut self, half_width: f64) -> f64 {
        (self.next_unit() * 2.0 - 1.0) * half_width
    }
}

fn agrees(original: f64, simplified: f64) -> bool {
    (original - simplified).abs() <= TOLERANCE * original.abs().max(1.0)
}

/// Returns the number of sample points at which the original was defined and the
/// simplified form was asserted to agree.
fn assert_value_preserving(source: &str, seed: u64) -> usize {
    let original = parse(source).unwrap_or_else(|error| panic!("parse {source:?}: {error}"));
    let simplified = simplify_expr(&original, &RewriteConfig::default())
        .unwrap_or_else(|error| panic!("simplify {source:?}: {error}"));

    // Simplification must never make an expression more expensive.
    assert!(
        expression_cost(&simplified) <= expression_cost(&original),
        "simplification increased cost for {source:?}"
    );

    let ordered_symbols: Vec<Identifier> = symbols(&original).into_iter().collect();
    let mut generator = Lcg::new(seed);
    let mut checked = 0usize;

    for _ in 0..SAMPLES_PER_EXPRESSION {
        let environment: Environment = ordered_symbols
            .iter()
            .map(|symbol| (symbol.clone(), generator.next_in_range(SAMPLE_RANGE)))
            .collect();

        // Only assert where the original is defined; documented rewrites may
        // widen the domain but must never disagree where the original has a
        // value.
        if let Ok(original_value) = evaluate(&original, &environment) {
            let simplified_value = evaluate(&simplified, &environment).unwrap_or_else(|error| {
                panic!(
                    "{source:?} original defined but simplified failed: {error} at {environment:?}"
                )
            });
            assert!(
                agrees(original_value, simplified_value),
                "value mismatch for {source:?}: original={original_value}, \
                 simplified={simplified_value} at {environment:?}"
            );
            checked += 1;
        }
    }

    checked
}

/// Every expression that must survive a soundness sweep. Chosen so that each
/// rewrite rule is exercised on its valid domain.
fn battery() -> Vec<&'static str> {
    vec![
        // Additive / multiplicative identities.
        "x + 0",
        "0 + x",
        "x - 0",
        "0 - x",
        "x * 1",
        "1 * x",
        "x * 0",
        "0 * x",
        // Cancellation (domain-widening but value-preserving where defined).
        "x - x",
        "x / x",
        "x / 1",
        "0 / x",
        // Power rules.
        "x ^ 1",
        "x ^ 0",
        "(x ^ 2) * (x ^ 3)",
        "(x ^ 2) ^ 3",
        "x ^ 2 + 0",
        // Log / exp inverses and products.
        "log(exp(x))",
        "exp(log(x))",
        "exp(x) * exp(y)",
        "log(exp(x)) + exp(log(y))",
        // Trigonometric parity and the Pythagorean identity.
        "sin(-x)",
        "cos(-x)",
        "sin(x) ^ 2 + cos(x) ^ 2",
        "sin(-x) + cos(-x)",
        // Negation and folding.
        "-(-x)",
        "2 * 3",
        "1 + 1",
        "2 * x + 3 * x - 5 * x",
        // Distributive factoring.
        "a * b + a * c",
        "a * c + b * c",
        // Combined, realistic messy laws.
        "0 + (x * 1) + (y - y)",
        "(a * b + a * c) * 1 + 0 - log(exp(d))",
        "sin(x) ^ 2 + cos(x) ^ 2 + (z * 0) + w / w",
        "exp(a) * exp(b) + (p * 1) / 1 - (q - q)",
    ]
}

#[test]
fn every_rewrite_preserves_value_on_its_domain() {
    let mut total_checks = 0usize;
    for (index, source) in battery().into_iter().enumerate() {
        let checks = assert_value_preserving(
            source,
            0x51ED_C0DE_u64 ^ (index as u64).wrapping_mul(2_654_435_761),
        );
        assert!(
            checks >= 16,
            "expression {source:?} was defined at too few sample points ({checks}); \
             the soundness check would be vacuous"
        );
        total_checks += checks;
    }
    // A visible, honest count of how much numerical evidence backs the claim.
    assert!(
        total_checks >= 5_000,
        "expected thousands of sampled soundness checks, got {total_checks}"
    );
    println!("asserted {total_checks} sampled value-agreement checks at tolerance {TOLERANCE:e}");
}

#[test]
fn simplification_is_deterministic_and_bit_identical() {
    let config = RewriteConfig::default();
    for source in battery() {
        let expression = parse(source).unwrap();
        let first = simplify_expr(&expression, &config).unwrap();
        let second = simplify_expr(&expression, &config).unwrap();
        // Structural identity, including identical f64 bit patterns via the
        // canonical string's full-precision formatting.
        assert_eq!(
            first.to_canonical_string(),
            second.to_canonical_string(),
            "non-deterministic simplification for {source:?}"
        );
    }
}

#[test]
fn already_minimal_expressions_are_unchanged() {
    let config = RewriteConfig::default();
    for source in ["x + y", "a * b", "sin(x)", "exp(x)", "x - y"] {
        let expression = parse(source).unwrap();
        let normalized = lawsynth_egraph::normalize(expression);
        let simplified = simplify_expr(&normalized, &config).unwrap();
        assert_eq!(
            simplified.to_canonical_string(),
            normalized.to_canonical_string(),
            "already-minimal expression changed: {source:?}"
        );
    }
}
