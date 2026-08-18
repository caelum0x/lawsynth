use lawsynth_core::Identifier;
use lawsynth_expr::{Expr, UnaryOperator};

use crate::{InvariantConfig, InvariantError};

/// A single labelled candidate function `φ_j(x)` in the parametrization
/// `H(x) = Σ_j c_j φ_j(x)`.
#[derive(Clone, Debug, PartialEq)]
pub struct BasisFunction {
    /// A stable, human-readable label such as `x^2`, `x*y`, or `cos(x)`.
    pub label: String,
    /// The expression tree evaluated and differentiated during detection.
    pub expression: Expr,
}

/// Builds the deterministic candidate library over `states`.
///
/// The library holds every monomial of total degree `1..=degree` (the constant
/// is deliberately excluded so `H = const` is never reported), optionally
/// followed by `sin(x_i)` and `cos(x_i)` for each state. Ordering is fixed:
/// monomials by ascending total degree in odometer order, then the trigonometric
/// terms in state order.
pub fn build_basis(
    states: &[Identifier],
    config: &InvariantConfig,
) -> Result<Vec<BasisFunction>, InvariantError> {
    let mut library = Vec::new();
    let mut exponents = vec![0usize; states.len()];
    for total_degree in 1..=config.degree {
        collect_monomials(states, total_degree, 0, &mut exponents, &mut library);
    }
    if config.include_trigonometric {
        for state in states {
            library.push(BasisFunction {
                label: format!("sin({})", state.as_str()),
                expression: Expr::unary(UnaryOperator::Sin, Expr::symbol(state.clone())),
            });
            library.push(BasisFunction {
                label: format!("cos({})", state.as_str()),
                expression: Expr::unary(UnaryOperator::Cos, Expr::symbol(state.clone())),
            });
        }
    }
    if library.is_empty() {
        return Err(InvariantError::EmptyLibrary);
    }
    Ok(library)
}

/// Enumerates monomials of exactly `remaining` total degree via a fixed-order
/// odometer over exponent vectors.
fn collect_monomials(
    states: &[Identifier],
    remaining: usize,
    index: usize,
    exponents: &mut [usize],
    library: &mut Vec<BasisFunction>,
) {
    if index + 1 == states.len() {
        exponents[index] = remaining;
        library.push(monomial(states, exponents));
        return;
    }
    // Descending exponent on the current variable gives a natural leading-term
    // order: `x`, `y`, then `x^2`, `x*y`, `y^2`, …
    for exponent in (0..=remaining).rev() {
        exponents[index] = exponent;
        collect_monomials(states, remaining - exponent, index + 1, exponents, library);
    }
}

/// Assembles one monomial as a repeated product of state symbols (which keeps
/// evaluation and differentiation exact for every real base) plus a clean label.
fn monomial(states: &[Identifier], exponents: &[usize]) -> BasisFunction {
    let mut expression = Expr::constant(1.0);
    let mut parts = Vec::new();
    for (state, &exponent) in states.iter().zip(exponents) {
        if exponent == 0 {
            continue;
        }
        for _ in 0..exponent {
            expression = Expr::product(expression, Expr::symbol(state.clone()));
        }
        if exponent == 1 {
            parts.push(state.as_str().to_owned());
        } else {
            parts.push(format!("{}^{}", state.as_str(), exponent));
        }
    }
    BasisFunction { label: parts.join("*"), expression: expression.simplify() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn excludes_the_constant_and_orders_monomials_by_degree() {
        let states = [id("x"), id("y")];
        let config = InvariantConfig { degree: 2, ..InvariantConfig::default() };
        let basis = build_basis(&states, &config).unwrap();
        let labels: Vec<&str> = basis.iter().map(|term| term.label.as_str()).collect();
        // Degree 1 then degree 2; no empty (constant) label anywhere.
        assert_eq!(labels, ["x", "y", "x^2", "x*y", "y^2"]);
        assert!(basis.iter().all(|term| !term.label.is_empty()));
    }

    #[test]
    fn appends_trigonometric_terms_when_requested() {
        let states = [id("x"), id("y")];
        let config = InvariantConfig {
            degree: 1,
            include_trigonometric: true,
            ..InvariantConfig::default()
        };
        let labels: Vec<String> =
            build_basis(&states, &config).unwrap().into_iter().map(|term| term.label).collect();
        assert_eq!(labels, ["x", "y", "sin(x)", "cos(x)", "sin(y)", "cos(y)"]);
    }
}
