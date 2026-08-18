//! Exact, deterministic substitution of a scalar parameter into an expression.
//!
//! The `lawsynth-expr` IR exposes differentiation and simplification but no
//! parameter binding, so this module supplies a small structural substitution:
//! every occurrence of the parameter symbol is replaced by a constant, leaving
//! the rest of the tree untouched. The result is a *parameter-free* field that
//! [`lawsynth_stability::analyze_stability`] accepts as an autonomous system.

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;

/// Returns a copy of `expression` with every `parameter` symbol replaced by the
/// constant `value`.
///
/// The traversal is a pure structural rewrite: it visits the tree in a fixed
/// pre-order, allocates a new tree, and never mutates the input. No folding or
/// simplification is applied, so the substitution is exact and reproducible —
/// downstream stages (Jacobian assembly, `simplify`) may fold as they see fit.
pub fn substitute(expression: &Expr, parameter: &Identifier, value: f64) -> Expr {
    match expression {
        Expr::Constant(constant) => Expr::constant(*constant),
        Expr::Symbol(identifier) => {
            if identifier == parameter {
                Expr::constant(value)
            } else {
                Expr::symbol(identifier.clone())
            }
        }
        Expr::Unary { operator, operand } => {
            Expr::unary(*operator, substitute(operand, parameter, value))
        }
        Expr::Binary { operator, left, right } => Expr::binary(
            *operator,
            substitute(left, parameter, value),
            substitute(right, parameter, value),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lawsynth_expr::{Environment, UnaryOperator, evaluate};

    fn id(name: &str) -> Identifier {
        Identifier::new(name).unwrap()
    }

    #[test]
    fn replaces_the_parameter_symbol_with_a_constant() {
        let mu = id("mu");
        let x = id("x");
        // f = mu * x - x^2
        let field = Expr::difference(
            Expr::product(Expr::symbol(mu.clone()), Expr::symbol(x.clone())),
            Expr::product(Expr::symbol(x.clone()), Expr::symbol(x.clone())),
        );
        let bound = substitute(&field, &mu, 0.5);
        // Only x remains free; evaluate at x = 2 => 0.5*2 - 4 = -3.
        let environment = Environment::from([(x.clone(), 2.0)]);
        assert_eq!(evaluate(&bound, &environment).unwrap(), -3.0);
    }

    #[test]
    fn leaves_other_symbols_and_structure_intact() {
        let mu = id("mu");
        let x = id("x");
        let field = Expr::unary(UnaryOperator::Sin, Expr::symbol(x.clone()));
        let bound = substitute(&field, &mu, 3.0);
        // No `mu` present, so the tree is structurally unchanged.
        assert_eq!(bound, field);
    }

    #[test]
    fn substitution_is_deterministic_bitwise() {
        let mu = id("mu");
        let x = id("x");
        let field = Expr::product(Expr::symbol(mu.clone()), Expr::symbol(x.clone()));
        let first = substitute(&field, &mu, 1.0 / 3.0);
        let second = substitute(&field, &mu, 1.0 / 3.0);
        assert_eq!(first.to_canonical_string(), second.to_canonical_string());
    }
}
