//! Human-readable rendering of scalar expressions and numbers.
//!
//! Unlike [`lawsynth_expr::print`], which emits a fully parenthesized,
//! interchange-oriented form with 17-digit constants, this module renders
//! precedence-aware, compactly formatted equations meant for people to read.

use lawsynth_core::Identifier;
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

// --- Python emitter ------------------------------------------------------

/// Formats a floating-point constant as a round-trippable Python literal.
///
/// Unlike [`format_number`], this preserves full precision so the generated
/// Python reproduces the world's dynamics exactly.
pub fn python_number(value: f64) -> String {
    if value == 0.0 {
        return "0.0".to_owned();
    }
    // Rust's `Debug` formatting for `f64` yields the shortest string that
    // round-trips, and always includes a decimal point or exponent, so the
    // literal is unambiguously a Python float.
    format!("{value:?}")
}

/// Renders an expression as Python arithmetic.
///
/// Each symbol is resolved through `resolve`, letting the caller map state
/// variables and parameters onto whatever Python bindings the generated
/// module uses (for example `state['x']` or `params['k']`).
pub fn render_python_expression(
    expression: &Expr,
    resolve: &dyn Fn(&Identifier) -> String,
) -> String {
    render_python(expression, 0, resolve)
}

fn render_python(
    expression: &Expr,
    parent_precedence: u8,
    resolve: &dyn Fn(&Identifier) -> String,
) -> String {
    let this_precedence = precedence(expression);
    let text = match expression {
        Expr::Constant(value) => python_number(*value),
        Expr::Symbol(id) => resolve(id),
        Expr::Unary { operator, operand } => match operator {
            UnaryOperator::Negate => format!("-{}", render_python(operand, PREC_ATOM, resolve)),
            UnaryOperator::Exp => format!("math.exp({})", render_python(operand, 0, resolve)),
            UnaryOperator::Log => format!("math.log({})", render_python(operand, 0, resolve)),
            UnaryOperator::Sin => format!("math.sin({})", render_python(operand, 0, resolve)),
            UnaryOperator::Cos => format!("math.cos({})", render_python(operand, 0, resolve)),
        },
        Expr::Binary { operator, left, right } => {
            let (symbol, precedence) = match operator {
                BinaryOperator::Add => ("+", PREC_ADD),
                BinaryOperator::Subtract => ("-", PREC_ADD),
                BinaryOperator::Multiply => ("*", PREC_MUL),
                BinaryOperator::Divide => ("/", PREC_MUL),
                BinaryOperator::Power => ("**", PREC_POW),
            };
            // Exponentiation is right-associative in Python; parenthesize both
            // sides one level tighter so a nested `(a**b)**c` keeps its meaning.
            let (left_precedence, right_precedence) = match operator {
                BinaryOperator::Power => (precedence + 1, precedence + 1),
                BinaryOperator::Subtract | BinaryOperator::Divide => (precedence, precedence + 1),
                _ => (precedence, precedence),
            };
            format!(
                "{} {symbol} {}",
                render_python(left, left_precedence, resolve),
                render_python(right, right_precedence, resolve)
            )
        }
    };
    if this_precedence < parent_precedence { format!("({text})") } else { text }
}

// --- C emitter -----------------------------------------------------------

/// Renders an expression as C `double` arithmetic.
///
/// Each symbol is resolved through `resolve`, letting the caller map state
/// variables and parameters onto whatever C bindings the generated source uses
/// (for example `state[0]` or the `K` macro). Powers become `pow(base, exp)`
/// (C has no `^` operator) and the transcendental unary operators map onto the
/// `<math.h>` functions `exp`, `log`, `sin`, and `cos`.
pub fn render_c_expression(expression: &Expr, resolve: &dyn Fn(&Identifier) -> String) -> String {
    render_c(expression, 0, resolve)
}

fn render_c(
    expression: &Expr,
    parent_precedence: u8,
    resolve: &dyn Fn(&Identifier) -> String,
) -> String {
    let this_precedence = precedence(expression);
    let text = match expression {
        Expr::Constant(value) => python_number(*value),
        Expr::Symbol(id) => resolve(id),
        Expr::Unary { operator, operand } => match operator {
            UnaryOperator::Negate => format!("-{}", render_c(operand, PREC_ATOM, resolve)),
            UnaryOperator::Exp => format!("exp({})", render_c(operand, 0, resolve)),
            UnaryOperator::Log => format!("log({})", render_c(operand, 0, resolve)),
            UnaryOperator::Sin => format!("sin({})", render_c(operand, 0, resolve)),
            UnaryOperator::Cos => format!("cos({})", render_c(operand, 0, resolve)),
        },
        // C has no exponent operator; `pow` is a self-delimiting call, so its
        // arguments never need outer parentheses.
        Expr::Binary { operator: BinaryOperator::Power, left, right } => {
            format!("pow({}, {})", render_c(left, 0, resolve), render_c(right, 0, resolve))
        }
        Expr::Binary { operator, left, right } => {
            let (symbol, precedence) = match operator {
                BinaryOperator::Add => ("+", PREC_ADD),
                BinaryOperator::Subtract => ("-", PREC_ADD),
                BinaryOperator::Multiply => ("*", PREC_MUL),
                BinaryOperator::Divide => ("/", PREC_MUL),
                BinaryOperator::Power => unreachable!("power handled above"),
            };
            let (left_precedence, right_precedence) = match operator {
                BinaryOperator::Subtract | BinaryOperator::Divide => (precedence, precedence + 1),
                _ => (precedence, precedence),
            };
            format!(
                "{} {symbol} {}",
                render_c(left, left_precedence, resolve),
                render_c(right, right_precedence, resolve)
            )
        }
    };
    if this_precedence < parent_precedence { format!("({text})") } else { text }
}

// --- MATLAB / Octave emitter ---------------------------------------------

/// Renders an expression as MATLAB/Octave arithmetic.
///
/// Each symbol is resolved through `resolve` (for example `state(1)` or a
/// parameter macro). Powers use the scalar `^` operator and the transcendental
/// unary operators map onto MATLAB's `exp`, `log` (natural log), `sin`, and
/// `cos`. Structure-preserving parentheses are emitted so the rendered form is
/// independent of MATLAB's operator associativity.
pub fn render_matlab_expression(
    expression: &Expr,
    resolve: &dyn Fn(&Identifier) -> String,
) -> String {
    render_matlab(expression, 0, resolve)
}

fn render_matlab(
    expression: &Expr,
    parent_precedence: u8,
    resolve: &dyn Fn(&Identifier) -> String,
) -> String {
    let this_precedence = precedence(expression);
    let text = match expression {
        Expr::Constant(value) => python_number(*value),
        Expr::Symbol(id) => resolve(id),
        Expr::Unary { operator, operand } => match operator {
            UnaryOperator::Negate => format!("-{}", render_matlab(operand, PREC_ATOM, resolve)),
            UnaryOperator::Exp => format!("exp({})", render_matlab(operand, 0, resolve)),
            UnaryOperator::Log => format!("log({})", render_matlab(operand, 0, resolve)),
            UnaryOperator::Sin => format!("sin({})", render_matlab(operand, 0, resolve)),
            UnaryOperator::Cos => format!("cos({})", render_matlab(operand, 0, resolve)),
        },
        Expr::Binary { operator, left, right } => {
            let (symbol, precedence) = match operator {
                BinaryOperator::Add => ("+", PREC_ADD),
                BinaryOperator::Subtract => ("-", PREC_ADD),
                BinaryOperator::Multiply => ("*", PREC_MUL),
                BinaryOperator::Divide => ("/", PREC_MUL),
                BinaryOperator::Power => ("^", PREC_POW),
            };
            // Parenthesize both operands of a non-associative operator one level
            // tighter so the rendered form preserves the expression tree
            // regardless of MATLAB's left-associative `^`.
            let (left_precedence, right_precedence) = match operator {
                BinaryOperator::Power => (precedence + 1, precedence + 1),
                BinaryOperator::Subtract | BinaryOperator::Divide => (precedence, precedence + 1),
                _ => (precedence, precedence),
            };
            format!(
                "{} {symbol} {}",
                render_matlab(left, left_precedence, resolve),
                render_matlab(right, right_precedence, resolve)
            )
        }
    };
    if this_precedence < parent_precedence { format!("({text})") } else { text }
}

// --- LaTeX emitter -------------------------------------------------------

/// Renders a continuous law as a LaTeX `\dot{target} = ...` equation body.
///
/// The returned string is a single row suitable for an `align*` environment
/// (without the trailing `\\`).
pub fn render_latex_law(target: &str, expression: &Expr) -> String {
    format!("\\dot{{{}}} &= {}", latex_symbol_name(target), render_latex_expression(expression))
}

/// Renders an expression as LaTeX math (no surrounding `$`).
pub fn render_latex_expression(expression: &Expr) -> String {
    render_latex(expression, 0)
}

fn latex_precedence(expression: &Expr) -> u8 {
    match expression {
        Expr::Binary { operator: BinaryOperator::Add | BinaryOperator::Subtract, .. } => PREC_ADD,
        Expr::Binary { operator: BinaryOperator::Multiply, .. } => PREC_MUL,
        // Division renders as a self-delimiting \frac, and powers/functions are
        // self-delimiting too, so they never need outer parentheses.
        _ => PREC_ATOM,
    }
}

fn render_latex(expression: &Expr, parent_precedence: u8) -> String {
    let this_precedence = latex_precedence(expression);
    let text = match expression {
        Expr::Constant(value) => format_number(*value),
        Expr::Symbol(id) => latex_symbol_name(id.as_str()),
        Expr::Unary { operator, operand } => match operator {
            UnaryOperator::Negate => format!("-{}", render_latex(operand, PREC_MUL)),
            UnaryOperator::Exp => format!("\\exp\\!\\left({}\\right)", render_latex(operand, 0)),
            UnaryOperator::Log => format!("\\ln\\!\\left({}\\right)", render_latex(operand, 0)),
            UnaryOperator::Sin => format!("\\sin\\!\\left({}\\right)", render_latex(operand, 0)),
            UnaryOperator::Cos => format!("\\cos\\!\\left({}\\right)", render_latex(operand, 0)),
        },
        Expr::Binary { operator, left, right } => render_latex_binary(*operator, left, right),
    };
    if this_precedence < parent_precedence { format!("\\left({text}\\right)") } else { text }
}

fn render_latex_binary(operator: BinaryOperator, left: &Expr, right: &Expr) -> String {
    match operator {
        BinaryOperator::Add => {
            format!("{} + {}", render_latex(left, PREC_ADD), render_latex(right, PREC_ADD))
        }
        BinaryOperator::Subtract => {
            format!("{} - {}", render_latex(left, PREC_ADD), render_latex(right, PREC_MUL))
        }
        BinaryOperator::Multiply => {
            // Render `-1 * rhs` as a leading minus, matching the readable renderer.
            if let Expr::Constant(value) = left {
                if *value == -1.0 {
                    return format!("-{}", render_latex(right, PREC_MUL));
                }
            }
            if let Expr::Constant(value) = right {
                if *value == -1.0 {
                    return format!("-{}", render_latex(left, PREC_MUL));
                }
            }
            format!("{} \\cdot {}", render_latex(left, PREC_MUL), render_latex(right, PREC_MUL))
        }
        BinaryOperator::Divide => {
            format!("\\frac{{{}}}{{{}}}", render_latex(left, 0), render_latex(right, 0))
        }
        BinaryOperator::Power => {
            format!("{}^{{{}}}", render_latex(left, PREC_ATOM), render_latex(right, 0))
        }
    }
}

/// Maps a small set of spelled-out Greek letter names onto LaTeX commands,
/// leaving all other identifiers unchanged.
fn latex_symbol_name(name: &str) -> String {
    const GREEK: &[&str] = &[
        "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "iota", "kappa",
        "lambda", "mu", "nu", "xi", "pi", "rho", "sigma", "tau", "phi", "chi", "psi", "omega",
    ];
    if GREEK.contains(&name) { format!("\\{name}") } else { name.to_owned() }
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn renders_python_arithmetic_with_symbol_resolution() {
        let expression = Expr::difference(
            Expr::product(Expr::symbol(Identifier::new("alpha").unwrap()), sym("x")),
            Expr::product(sym("x"), sym("y")),
        );
        let rendered = render_python_expression(&expression, &|id| match id.as_str() {
            "alpha" => "params['alpha']".to_owned(),
            other => format!("state['{other}']"),
        });
        assert_eq!(rendered, "params['alpha'] * state['x'] - state['x'] * state['y']");
    }

    #[test]
    fn python_numbers_round_trip() {
        assert_eq!(python_number(1.0), "1.0");
        assert_eq!(python_number(-2.6666666666666665), "-2.6666666666666665");
        assert_eq!(python_number(0.0), "0.0");
    }

    #[test]
    fn renders_latex_with_greek_and_fractions() {
        let expression = Expr::quotient(
            Expr::product(Expr::symbol(Identifier::new("sigma").unwrap()), sym("y")),
            sym("x"),
        );
        assert_eq!(render_latex_expression(&expression), "\\frac{\\sigma \\cdot y}{x}");
    }

    #[test]
    fn renders_latex_law_with_dot_notation() {
        let law = render_latex_law("x", &Expr::product(Expr::constant(-1.0), sym("x")));
        assert_eq!(law, "\\dot{x} &= -x");
    }

    #[test]
    fn renders_c_arithmetic_with_pow_and_precedence() {
        // (alpha * x - x*y) with a power term x^2.
        let expression = Expr::difference(
            Expr::product(Expr::symbol(Identifier::new("alpha").unwrap()), sym("x")),
            Expr::binary(BinaryOperator::Power, sym("x"), Expr::constant(2.0)),
        );
        let rendered = render_c_expression(&expression, &|id| match id.as_str() {
            "alpha" => "P_alpha".to_owned(),
            other => format!("state[{other}]"),
        });
        assert_eq!(rendered, "P_alpha * state[x] - pow(state[x], 2.0)");
    }

    #[test]
    fn renders_c_keeps_parentheses_and_math_calls() {
        let expression =
            Expr::product(Expr::sum(sym("x"), sym("y")), Expr::unary(UnaryOperator::Sin, sym("z")));
        let rendered = render_c_expression(&expression, &|id| id.as_str().to_owned());
        assert_eq!(rendered, "(x + y) * sin(z)");
    }

    #[test]
    fn renders_matlab_arithmetic_with_caret_power() {
        let expression = Expr::difference(
            Expr::product(Expr::symbol(Identifier::new("alpha").unwrap()), sym("x")),
            Expr::binary(BinaryOperator::Power, sym("x"), Expr::constant(2.0)),
        );
        let rendered = render_matlab_expression(&expression, &|id| match id.as_str() {
            "alpha" => "P_alpha".to_owned(),
            other => format!("state({other})"),
        });
        assert_eq!(rendered, "P_alpha * state(x) - state(x) ^ 2.0");
    }

    #[test]
    fn renders_matlab_preserves_structure_with_parentheses() {
        let expression = Expr::product(Expr::sum(sym("x"), sym("y")), sym("z"));
        let rendered = render_matlab_expression(&expression, &|id| id.as_str().to_owned());
        assert_eq!(rendered, "(x + y) * z");
    }
}
