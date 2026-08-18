//! Shared helpers for the round-trip integration tests.
//!
//! The core helper decomposes a discovered law expression into a
//! `monomial -> coefficient` map so recovery can be asserted exactly, both on
//! term structure (which monomials are active) and on coefficient value.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use lawsynth_core::Identifier;
use lawsynth_discovery::discover;
use lawsynth_domains::DomainPreset;
use lawsynth_expr::{BinaryOperator, Expr, UnaryOperator};

/// A monomial key: the exponent of each state variable, in the preset's own
/// (reference) variable order.
pub type Exponents = Vec<u32>;

/// Decomposes a discovered polynomial law into `exponents -> coefficient`.
///
/// The discovery pipeline builds each law as a sum of `constant · monomial`
/// terms, so this walks the additive structure and folds every product into a
/// scalar coefficient and a per-variable exponent vector. Panics on any node
/// shape a polynomial law should never contain (e.g. a stray transcendental),
/// which would itself be a recovery failure worth surfacing.
pub fn monomials(expression: &Expr, variables: &[Identifier]) -> BTreeMap<Exponents, f64> {
    let mut accumulator = BTreeMap::new();
    collect_additive(expression, 1.0, variables, &mut accumulator);
    // Selected terms are all real, but fold sums that cancel to ~0 are dropped so
    // the support reflects genuinely active monomials only.
    accumulator.retain(|_, coefficient| coefficient.abs() > 1e-9);
    accumulator
}

fn collect_additive(
    expression: &Expr,
    sign: f64,
    variables: &[Identifier],
    accumulator: &mut BTreeMap<Exponents, f64>,
) {
    match expression {
        Expr::Binary { operator: BinaryOperator::Add, left, right } => {
            collect_additive(left, sign, variables, accumulator);
            collect_additive(right, sign, variables, accumulator);
        }
        Expr::Binary { operator: BinaryOperator::Subtract, left, right } => {
            collect_additive(left, sign, variables, accumulator);
            collect_additive(right, -sign, variables, accumulator);
        }
        product => {
            let (coefficient, exponents) = fold_product(product, variables);
            *accumulator.entry(exponents).or_insert(0.0) += sign * coefficient;
        }
    }
}

fn fold_product(expression: &Expr, variables: &[Identifier]) -> (f64, Exponents) {
    let mut coefficient = 1.0;
    let mut exponents = vec![0u32; variables.len()];
    fold(expression, &mut coefficient, &mut exponents, variables);
    (coefficient, exponents)
}

fn fold(expression: &Expr, coefficient: &mut f64, exponents: &mut [u32], variables: &[Identifier]) {
    match expression {
        Expr::Constant(value) => *coefficient *= value,
        Expr::Symbol(symbol) => {
            let index = variables
                .iter()
                .position(|variable| variable == symbol)
                .expect("discovered symbol is a declared state variable");
            exponents[index] += 1;
        }
        Expr::Binary { operator: BinaryOperator::Multiply, left, right } => {
            fold(left, coefficient, exponents, variables);
            fold(right, coefficient, exponents, variables);
        }
        Expr::Binary { operator: BinaryOperator::Power, left, right } => match right.as_ref() {
            Expr::Constant(power) if *power >= 0.0 && power.fract() == 0.0 => {
                for _ in 0..(*power as u32) {
                    fold(left, coefficient, exponents, variables);
                }
            }
            other => panic!("unexpected power exponent in discovered law: {other:?}"),
        },
        Expr::Unary { operator: UnaryOperator::Negate, operand } => {
            *coefficient = -*coefficient;
            fold(operand, coefficient, exponents, variables);
        }
        other => panic!("unexpected node in discovered polynomial law: {other:?}"),
    }
}

/// Runs discovery with the preset's own configuration and returns the recovered
/// `state -> (monomial -> coefficient)` map for the sparse candidate.
pub fn recovered_laws(preset: &DomainPreset) -> BTreeMap<Identifier, BTreeMap<Exponents, f64>> {
    let data = preset.reference().trajectory();
    let result = discover(&data, &preset.discovery_config()).expect("discovery succeeds");
    let laws = result.candidates[0].world.laws();
    preset
        .state_variables()
        .iter()
        .map(|state| (state.clone(), monomials(&laws[state].expression, preset.state_variables())))
        .collect()
}

/// The reference law's `monomial -> coefficient` map for one state variable.
pub fn reference_monomials(preset: &DomainPreset, state: &Identifier) -> BTreeMap<Exponents, f64> {
    let mut reference = BTreeMap::new();
    for term in preset.reference().law(state).expect("state has a reference law").terms() {
        *reference.entry(term.exponents.clone()).or_insert(0.0) += term.coefficient;
    }
    reference.retain(|_, coefficient| coefficient.abs() > 1e-12);
    reference
}

/// Asserts full round-trip recovery: the discovered law for every state has the
/// same active-monomial support as the reference law, and every coefficient
/// matches within `coefficient_tolerance`.
pub fn assert_round_trip(preset: &DomainPreset, coefficient_tolerance: f64) {
    let recovered = recovered_laws(preset);
    for state in preset.state_variables() {
        let discovered = &recovered[state];
        let reference = reference_monomials(preset, state);

        let discovered_support: BTreeSet<Exponents> = discovered.keys().cloned().collect();
        let reference_support: BTreeSet<Exponents> = reference.keys().cloned().collect();
        assert_eq!(
            discovered_support, reference_support,
            "term structure mismatch for d{state}/dt: discovered {discovered_support:?}, \
             reference {reference_support:?}",
        );

        for (exponents, &expected) in &reference {
            let got = discovered[exponents];
            assert!(
                (got - expected).abs() <= coefficient_tolerance,
                "coefficient mismatch for d{state}/dt monomial {exponents:?}: got {got}, \
                 expected {expected} (tolerance {coefficient_tolerance:e})",
            );
        }
    }
}
