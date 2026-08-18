use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_jacobian::differentiate;

use crate::InvariantError;

/// Builds the symbolic Lie derivative `L_f φ = Σ_i (∂φ/∂x_i) · f_i`.
///
/// `resolved` pairs each state `x_i` with its field expression `f_i` in state
/// order. The result is conservatively simplified for cheaper evaluation; it is
/// mathematically the directional derivative of `φ` along the flow, whose
/// vanishing everywhere is exactly the conservation condition.
pub fn lie_derivative(
    phi: &Expr,
    resolved: &[(&Identifier, &Expr)],
) -> Result<Expr, InvariantError> {
    let mut accumulator = Expr::constant(0.0);
    for (state, field) in resolved {
        let partial = differentiate(phi, state)?;
        let term = Expr::product(partial, (*field).clone());
        accumulator = Expr::sum(accumulator, term);
    }
    Ok(accumulator.simplify())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lawsynth_expr::{Environment, UnaryOperator, evaluate};

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn energy_of_the_harmonic_oscillator_is_conserved_pointwise() {
        // ẋ = y, ẏ = -x; H = x^2 + y^2 has L_f H = 0 everywhere.
        let x = id("x");
        let y = id("y");
        let fx = Expr::symbol(y.clone());
        let fy = Expr::unary(UnaryOperator::Negate, Expr::symbol(x.clone()));
        let resolved = [(&x, &fx), (&y, &fy)];
        let energy = Expr::sum(
            Expr::product(Expr::symbol(x.clone()), Expr::symbol(x.clone())),
            Expr::product(Expr::symbol(y.clone()), Expr::symbol(y.clone())),
        );
        let lie = lie_derivative(&energy, &resolved).unwrap();
        for (sx, sy) in [(0.3, 1.1), (-0.7, 0.2), (2.0, -1.5)] {
            let environment = Environment::from([(x.clone(), sx), (y.clone(), sy)]);
            assert!(evaluate(&lie, &environment).unwrap().abs() < 1e-12);
        }
    }

    #[test]
    fn a_non_invariant_has_a_nonzero_lie_derivative() {
        // For the same flow, H = x is not conserved: L_f x = y.
        let x = id("x");
        let y = id("y");
        let fx = Expr::symbol(y.clone());
        let fy = Expr::unary(UnaryOperator::Negate, Expr::symbol(x.clone()));
        let resolved = [(&x, &fx), (&y, &fy)];
        let lie = lie_derivative(&Expr::symbol(x.clone()), &resolved).unwrap();
        let environment = Environment::from([(x, 0.0), (y, 1.7)]);
        assert!((evaluate(&lie, &environment).unwrap() - 1.7).abs() < 1e-12);
    }
}
