use lawsynth_core::Identifier;
use lawsynth_expr::{BinaryOperator, Expr, UnaryOperator, print};

use crate::error::JacobianError;

/// Symbolically differentiates `expr` with respect to the scalar symbol `wrt`.
///
/// The result is an unsimplified derivative expression; callers that want a
/// compact form apply [`Expr::simplify`]. Every symbol other than `wrt` — model
/// parameters, constants, other states — is treated as independent of `wrt`, so
/// its derivative is zero. All node kinds in the current IR are supported; see
/// [`JacobianError::UnsupportedDerivative`] for the single closed-form gap.
pub fn differentiate(expr: &Expr, wrt: &Identifier) -> Result<Expr, JacobianError> {
    match expr {
        Expr::Constant(_) => Ok(Expr::constant(0.0)),
        Expr::Symbol(identifier) => Ok(Expr::constant(if identifier == wrt { 1.0 } else { 0.0 })),
        Expr::Unary { operator, operand } => differentiate_unary(*operator, operand, wrt),
        Expr::Binary { operator, left, right } => differentiate_binary(*operator, left, right, wrt),
    }
}

fn differentiate_unary(
    operator: UnaryOperator,
    operand: &Expr,
    wrt: &Identifier,
) -> Result<Expr, JacobianError> {
    let inner = differentiate(operand, wrt)?;
    let derivative = match operator {
        // d(-u) = -u'
        UnaryOperator::Negate => Expr::unary(UnaryOperator::Negate, inner),
        // d(exp(u)) = exp(u) * u'
        UnaryOperator::Exp => {
            Expr::product(Expr::unary(UnaryOperator::Exp, operand.clone()), inner)
        }
        // d(log(u)) = u' / u
        UnaryOperator::Log => Expr::quotient(inner, operand.clone()),
        // d(sin(u)) = cos(u) * u'
        UnaryOperator::Sin => {
            Expr::product(Expr::unary(UnaryOperator::Cos, operand.clone()), inner)
        }
        // d(cos(u)) = -(sin(u) * u')
        UnaryOperator::Cos => Expr::unary(
            UnaryOperator::Negate,
            Expr::product(Expr::unary(UnaryOperator::Sin, operand.clone()), inner),
        ),
    };
    Ok(derivative)
}

fn differentiate_binary(
    operator: BinaryOperator,
    left: &Expr,
    right: &Expr,
    wrt: &Identifier,
) -> Result<Expr, JacobianError> {
    match operator {
        // d(l + r) = l' + r'
        BinaryOperator::Add => Ok(Expr::sum(differentiate(left, wrt)?, differentiate(right, wrt)?)),
        // d(l - r) = l' - r'
        BinaryOperator::Subtract => {
            Ok(Expr::difference(differentiate(left, wrt)?, differentiate(right, wrt)?))
        }
        // Product rule: d(l * r) = l'*r + l*r'
        BinaryOperator::Multiply => Ok(Expr::sum(
            Expr::product(differentiate(left, wrt)?, right.clone()),
            Expr::product(left.clone(), differentiate(right, wrt)?),
        )),
        // Quotient rule: d(l / r) = (l'*r - l*r') / r^2
        BinaryOperator::Divide => Ok(Expr::quotient(
            Expr::difference(
                Expr::product(differentiate(left, wrt)?, right.clone()),
                Expr::product(left.clone(), differentiate(right, wrt)?),
            ),
            Expr::product(right.clone(), right.clone()),
        )),
        BinaryOperator::Power => differentiate_power(left, right, wrt),
    }
}

/// Differentiates a power `base ^ exponent`, choosing the rule that keeps the
/// result valid over the widest real domain.
fn differentiate_power(
    base: &Expr,
    exponent: &Expr,
    wrt: &Identifier,
) -> Result<Expr, JacobianError> {
    match (base, exponent) {
        // Constant exponent c: d(f^c) = c * f^(c-1) * f'. Preferred whenever it
        // applies because it never introduces log(base) and so stays correct for
        // negative bases (e.g. d/dx x^2 at x < 0).
        (_, Expr::Constant(power)) => {
            let reduced =
                Expr::binary(BinaryOperator::Power, base.clone(), Expr::constant(power - 1.0));
            Ok(Expr::product(
                Expr::product(Expr::constant(*power), reduced),
                differentiate(base, wrt)?,
            ))
        }
        // Constant base b > 0: d(b^g) = b^g * ln(b) * g'.
        (Expr::Constant(value), _) => {
            if *value > 0.0 {
                let power = Expr::binary(BinaryOperator::Power, base.clone(), exponent.clone());
                Ok(Expr::product(
                    Expr::product(power, Expr::constant(value.ln())),
                    differentiate(exponent, wrt)?,
                ))
            } else {
                Err(JacobianError::UnsupportedDerivative {
                    node: print(&Expr::binary(
                        BinaryOperator::Power,
                        base.clone(),
                        exponent.clone(),
                    )),
                    reason: "b^g with a non-positive constant base and a variable exponent has no \
                             real closed-form derivative (would require log of a non-positive base)",
                })
            }
        }
        // General f^g: d = f^g * (g' * ln(f) + g * f'/f). Mathematically correct
        // and numerically valid wherever the base is positive.
        _ => {
            let power = Expr::binary(BinaryOperator::Power, base.clone(), exponent.clone());
            let log_term = Expr::product(
                differentiate(exponent, wrt)?,
                Expr::unary(UnaryOperator::Log, base.clone()),
            );
            let ratio_term = Expr::product(
                exponent.clone(),
                Expr::quotient(differentiate(base, wrt)?, base.clone()),
            );
            Ok(Expr::product(power, Expr::sum(log_term, ratio_term)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lawsynth_expr::{Environment, evaluate};

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    fn x() -> Identifier {
        id("x")
    }

    fn diff_simplified(expr: &Expr, wrt: &Identifier) -> Expr {
        differentiate(expr, wrt).unwrap().simplify()
    }

    fn eval_at(expr: &Expr, symbol: &str, value: f64) -> f64 {
        let environment = Environment::from([(id(symbol), value)]);
        evaluate(expr, &environment).unwrap()
    }

    #[test]
    fn constant_derivative_is_zero() {
        assert_eq!(diff_simplified(&Expr::constant(7.0), &x()), Expr::constant(0.0));
    }

    #[test]
    fn symbol_derivative_is_one_or_zero() {
        assert_eq!(diff_simplified(&Expr::symbol(x()), &x()), Expr::constant(1.0));
        assert_eq!(diff_simplified(&Expr::symbol(id("y")), &x()), Expr::constant(0.0));
    }

    #[test]
    fn negate_rule() {
        // d(-x) = -1
        assert_eq!(
            diff_simplified(&Expr::unary(UnaryOperator::Negate, Expr::symbol(x())), &x()),
            Expr::constant(-1.0)
        );
    }

    #[test]
    fn exp_rule_matches_itself() {
        // d(exp(x)) = exp(x)
        let derivative = diff_simplified(&Expr::unary(UnaryOperator::Exp, Expr::symbol(x())), &x());
        assert!((eval_at(&derivative, "x", 1.3) - 1.3_f64.exp()).abs() < 1e-12);
    }

    #[test]
    fn log_rule_is_reciprocal() {
        // d(log(x)) = 1/x
        let derivative = diff_simplified(&Expr::unary(UnaryOperator::Log, Expr::symbol(x())), &x());
        assert!((eval_at(&derivative, "x", 4.0) - 0.25).abs() < 1e-12);
    }

    #[test]
    fn sin_rule_is_cos() {
        let derivative = diff_simplified(&Expr::unary(UnaryOperator::Sin, Expr::symbol(x())), &x());
        assert!((eval_at(&derivative, "x", 0.7) - 0.7_f64.cos()).abs() < 1e-12);
    }

    #[test]
    fn cos_rule_is_negative_sin() {
        let derivative = diff_simplified(&Expr::unary(UnaryOperator::Cos, Expr::symbol(x())), &x());
        assert!((eval_at(&derivative, "x", 0.7) + 0.7_f64.sin()).abs() < 1e-12);
    }

    #[test]
    fn product_rule() {
        // d(x * sin(x)) = sin(x) + x cos(x)
        let expr =
            Expr::product(Expr::symbol(x()), Expr::unary(UnaryOperator::Sin, Expr::symbol(x())));
        let derivative = diff_simplified(&expr, &x());
        let expected = 0.5_f64.sin() + 0.5 * 0.5_f64.cos();
        assert!((eval_at(&derivative, "x", 0.5) - expected).abs() < 1e-12);
    }

    #[test]
    fn quotient_rule() {
        // d((x^2) / (x + 1)) evaluated numerically
        let numerator = Expr::binary(BinaryOperator::Power, Expr::symbol(x()), Expr::constant(2.0));
        let denominator = Expr::sum(Expr::symbol(x()), Expr::constant(1.0));
        let derivative = diff_simplified(&Expr::quotient(numerator, denominator), &x());
        // (2x(x+1) - x^2) / (x+1)^2 at x=2 -> (4*3 - 4)/9 = 8/9
        assert!((eval_at(&derivative, "x", 2.0) - 8.0 / 9.0).abs() < 1e-12);
    }

    #[test]
    fn power_constant_exponent_handles_negative_base() {
        // d(x^3) = 3 x^2, valid at x = -2 (would be NaN with a log-based rule).
        let expr = Expr::binary(BinaryOperator::Power, Expr::symbol(x()), Expr::constant(3.0));
        let derivative = diff_simplified(&expr, &x());
        assert!((eval_at(&derivative, "x", -2.0) - 12.0).abs() < 1e-12);
    }

    #[test]
    fn power_constant_base_uses_log() {
        // d(2^x) = 2^x ln 2
        let expr = Expr::binary(BinaryOperator::Power, Expr::constant(2.0), Expr::symbol(x()));
        let derivative = diff_simplified(&expr, &x());
        let expected = 8.0 * 2.0_f64.ln();
        assert!((eval_at(&derivative, "x", 3.0) - expected).abs() < 1e-12);
    }

    #[test]
    fn chain_rule_through_composition() {
        // d(sin(x^2)) = cos(x^2) * 2x
        let inner = Expr::binary(BinaryOperator::Power, Expr::symbol(x()), Expr::constant(2.0));
        let expr = Expr::unary(UnaryOperator::Sin, inner);
        let derivative = diff_simplified(&expr, &x());
        let value: f64 = 1.1;
        let expected = (value * value).cos() * 2.0 * value;
        assert!((eval_at(&derivative, "x", value) - expected).abs() < 1e-12);
    }

    #[test]
    fn unsupported_power_reports_error() {
        // d((-2)^x): non-positive constant base with a variable exponent.
        let expr = Expr::binary(BinaryOperator::Power, Expr::constant(-2.0), Expr::symbol(x()));
        let error = differentiate(&expr, &x()).unwrap_err();
        assert!(matches!(error, JacobianError::UnsupportedDerivative { .. }));
    }
}
