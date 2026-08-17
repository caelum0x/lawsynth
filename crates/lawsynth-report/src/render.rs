//! Human-readable rendering of scalar expressions and numbers.
//!
//! Unlike [`lawsynth_expr::print`], which emits a fully parenthesized,
//! interchange-oriented form with 17-digit constants, this module renders
//! precedence-aware, compactly formatted equations meant for people to read.

use lawsynth_expr::{BinaryOperator, Expr, UnaryOperator};

const PREC_ADD: u8 = 1;
const PREC_MUL: u8 = 2;
const PREC_POW: u8 = 3;
const PREC_ATOM: u8 = 4;

/// Formats a floating-point constant as a short, readable decimal.
///
/// Values in a human-friendly magnitude window are shown as trimmed fixed
/// decimals; very large or very small magnitudes fall back to scientific
/// notation. The output is deterministic for a given input.
pub fn format_number(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    if !value.is_finite() {
        return format!("{value}");
    }
    let magnitude = value.abs();
    if (1e-4..1e7).contains(&magnitude) {
        let fixed = format!("{value:.6}");
        let trimmed = fixed.trim_end_matches('0').trim_end_matches('.');
        trimmed.to_owned()
    } else {
        format!("{value:.4e}")
    }
}

/// Renders an expression as a readable, precedence-minimized string.
pub fn render_expression(expression: &Expr) -> String {
    render(expression, 0)
}

/// Renders a continuous law as `d{target}/dt = {expression}`.
pub fn render_continuous_law(target: &str, expression: &Expr) -> String {
    format!("d{target}/dt = {}", render_expression(expression))
}

/// Renders a discrete law as `{target}[t+1] = {expression}`.
pub fn render_discrete_law(target: &str, expression: &Expr) -> String {
    format!("{target}[t+1] = {}", render_expression(expression))
}

fn precedence(expression: &Expr) -> u8 {
    match expression {
        Expr::Constant(_) | Expr::Symbol(_) => PREC_ATOM,
        Expr::Unary { .. } => PREC_ATOM,
        Expr::Binary { operator, .. } => match operator {
            BinaryOperator::Add | BinaryOperator::Subtract => PREC_ADD,
            BinaryOperator::Multiply | BinaryOperator::Divide => PREC_MUL,
            BinaryOperator::Power => PREC_POW,
        },
    }
}

fn render(expression: &Expr, parent_precedence: u8) -> String {
    let this_precedence = precedence(expression);
    let text = match expression {
        Expr::Constant(value) => format_number(*value),
        Expr::Symbol(id) => id.as_str().to_owned(),
        Expr::Unary { operator, operand } => render_unary(*operator, operand),
        Expr::Binary { operator, left, right } => render_binary(*operator, left, right),
    };
    if this_precedence < parent_precedence { format!("({text})") } else { text }
}

fn render_unary(operator: UnaryOperator, operand: &Expr) -> String {
    match operator {
        UnaryOperator::Negate => format!("-{}", render(operand, PREC_ATOM)),
        UnaryOperator::Exp => format!("exp({})", render(operand, 0)),
        UnaryOperator::Log => format!("log({})", render(operand, 0)),
        UnaryOperator::Sin => format!("sin({})", render(operand, 0)),
        UnaryOperator::Cos => format!("cos({})", render(operand, 0)),
    }
}

fn render_binary(operator: BinaryOperator, left: &Expr, right: &Expr) -> String {
    // Render `-1 * rhs` and `rhs * -1` as a leading minus for readability.
    if operator == BinaryOperator::Multiply {
        if let Expr::Constant(value) = left {
            if *value == -1.0 {
                return format!("-{}", render(right, PREC_MUL));
            }
        }
        if let Expr::Constant(value) = right {
            if *value == -1.0 {
                return format!("-{}", render(left, PREC_MUL));
            }
        }
    }
    let (symbol, precedence) = match operator {
        BinaryOperator::Add => ("+", PREC_ADD),
        BinaryOperator::Subtract => ("-", PREC_ADD),
        BinaryOperator::Multiply => ("*", PREC_MUL),
        BinaryOperator::Divide => ("/", PREC_MUL),
        BinaryOperator::Power => ("^", PREC_POW),
    };
    // The right operand binds one level tighter for non-associative operators so
    // that `a - (b - c)` and `a / (b / c)` keep their parentheses.
    let right_precedence = match operator {
        BinaryOperator::Subtract | BinaryOperator::Divide | BinaryOperator::Power => precedence + 1,
        _ => precedence,
    };
    format!("{} {symbol} {}", render(left, precedence), render(right, right_precedence))
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;

    use super::*;

    fn sym(name: &str) -> Expr {
        Expr::symbol(Identifier::new(name).unwrap())
    }

    #[test]
    fn formats_short_decimals() {
        assert_eq!(format_number(0.5), "0.5");
        assert_eq!(format_number(-0.30000000001), "-0.3");
        assert_eq!(format_number(0.0), "0");
        assert_eq!(format_number(12.0), "12");
    }

    #[test]
    fn renders_precedence_minimized_sum_of_products() {
        let expression = Expr::sum(
            Expr::product(Expr::constant(0.5), sym("y")),
            Expr::product(Expr::constant(-1.0), sym("x")),
        );
        assert_eq!(render_expression(&expression), "0.5 * y + -x");
    }

    #[test]
    fn keeps_parentheses_where_precedence_requires() {
        let expression = Expr::product(Expr::sum(sym("x"), sym("y")), sym("z"));
        assert_eq!(render_expression(&expression), "(x + y) * z");
    }

    #[test]
    fn renders_continuous_law_heading() {
        assert_eq!(
            render_continuous_law("x", &Expr::product(Expr::constant(2.0), sym("x"))),
            "dx/dt = 2 * x"
        );
    }
}
