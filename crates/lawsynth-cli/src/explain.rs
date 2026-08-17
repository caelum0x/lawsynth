//! `lawsynth explain` — plain-language + structured explanation of a world.

use std::fmt::Write;

use lawsynth_bundle::read_world;
use lawsynth_core::Identifier;
use lawsynth_expr::{BinaryOperator, Expr, UnaryOperator};
use lawsynth_report::{format_number, render_continuous_law, render_expression};
use lawsynth_world::{VariableRole, World, expression_symbols};

/// Help text for `lawsynth explain`.
pub fn help() -> String {
    "lawsynth explain WORLD.lsworld\n\n\
Prints a plain-language and structured explanation of a world: what each law \
says, the variables and parameters it uses, and its dimensionality/complexity."
        .to_owned()
}

/// Runs the `explain` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    let Some(bundle) = arguments.first() else {
        return Err(help());
    };
    if bundle == "--help" || bundle == "-h" {
        return Ok(help());
    }
    if arguments.len() != 1 {
        return Err(help());
    }
    let world = read_world(bundle).map_err(|error| error.to_string())?;
    Ok(explain(&world))
}

fn explain(world: &World) -> String {
    let mut out = String::new();
    let states = world.state_ids().count();
    let total_complexity: usize =
        world.laws().values().map(|law| complexity(&law.expression)).sum();

    let _ = writeln!(out, "World summary");
    let _ = writeln!(
        out,
        "  {} state variable(s), {} variable(s) total, {} parameter(s)",
        states,
        world.variables().len(),
        world.parameters().len()
    );
    let _ = writeln!(
        out,
        "  dimensionality: {states}-dimensional  |  total complexity: {total_complexity} AST node(s)"
    );
    out.push('\n');

    let _ = writeln!(out, "Laws");
    for (target, law) in world.laws() {
        let _ = writeln!(out, "  {}", render_continuous_law(target.as_str(), &law.expression));
        for sentence in describe_law(target, &law.expression) {
            let _ = writeln!(out, "    - {sentence}");
        }
        let reads = expression_symbols(&law.expression);
        if !reads.is_empty() {
            let names: Vec<&str> = reads.iter().map(Identifier::as_str).collect();
            let _ = writeln!(out, "    reads: {}", names.join(", "));
        }
        out.push('\n');
    }

    let _ = writeln!(out, "Variables");
    for variable in world.variables().values() {
        let unit = variable.unit.as_ref().map(|unit| unit.canonical()).unwrap_or("dimensionless");
        let _ = writeln!(
            out,
            "  {:<16} {:<10} [{}]",
            variable.id.as_str(),
            role_name(variable.role),
            unit
        );
    }
    out.push('\n');

    if world.parameters().is_empty() {
        let _ = writeln!(out, "Parameters\n  (none)");
    } else {
        let _ = writeln!(out, "Parameters");
        for parameter in world.parameters().values() {
            let unit =
                parameter.unit.as_ref().map(|unit| unit.canonical()).unwrap_or("dimensionless");
            let _ = writeln!(
                out,
                "  {:<16} = {:<14} [{}]",
                parameter.id.as_str(),
                format_number(parameter.value),
                unit
            );
        }
    }
    out.push('\n');

    let _ = writeln!(out, "Notes");
    let _ = writeln!(out, "  Regime and dependency-hypothesis metadata are produced by");
    let _ = writeln!(
        out,
        "  `discover --regimes` / `--causal` and are not stored in the world bundle."
    );
    out
}

fn role_name(role: VariableRole) -> &'static str {
    match role {
        VariableRole::State => "state",
        VariableRole::Control => "control",
        VariableRole::Exogenous => "exogenous",
        VariableRole::Observed => "observed",
        VariableRole::Latent => "latent",
        VariableRole::Derived => "derived",
    }
}

fn complexity(expression: &Expr) -> usize {
    match expression {
        Expr::Constant(_) | Expr::Symbol(_) => 1,
        Expr::Unary { operand, .. } => 1 + complexity(operand),
        Expr::Binary { left, right, .. } => 1 + complexity(left) + complexity(right),
    }
}

/// Produces one plain-language sentence per additive term of a law.
fn describe_law(target: &Identifier, expression: &Expr) -> Vec<String> {
    let mut terms = Vec::new();
    flatten_terms(expression, 1.0, &mut terms);
    if terms.is_empty() {
        return vec![format!("{} stays constant.", target.as_str())];
    }
    terms.into_iter().map(|(sign, term)| describe_term(target, sign, &term)).collect()
}

/// Flattens a sum/difference tree into signed leaf terms.
fn flatten_terms(expression: &Expr, sign: f64, terms: &mut Vec<(f64, Expr)>) {
    match expression {
        Expr::Binary { operator: BinaryOperator::Add, left, right } => {
            flatten_terms(left, sign, terms);
            flatten_terms(right, sign, terms);
        }
        Expr::Binary { operator: BinaryOperator::Subtract, left, right } => {
            flatten_terms(left, sign, terms);
            flatten_terms(right, -sign, terms);
        }
        Expr::Unary { operator: UnaryOperator::Negate, operand } => {
            flatten_terms(operand, -sign, terms);
        }
        other => terms.push((sign, other.clone())),
    }
}

fn describe_term(target: &Identifier, sign: f64, term: &Expr) -> String {
    let (coefficient, symbolic) = strip_coefficient(term);
    let net = sign * coefficient;
    let magnitude = net.abs();
    let target = target.as_str();
    match symbolic {
        None => {
            let word = if net >= 0.0 { "gains" } else { "loses" };
            format!("{target} {word} a constant {} per unit time.", format_number(magnitude))
        }
        Some(factor) => {
            let direction = if net >= 0.0 { "increases" } else { "decreases" };
            let rendered = render_expression(&factor);
            if (magnitude - 1.0).abs() < f64::EPSILON {
                format!("{target} {direction} in proportion to {rendered}.")
            } else {
                format!(
                    "{target} {direction} in proportion to {rendered} (rate {}).",
                    format_number(magnitude)
                )
            }
        }
    }
}

/// Separates a multiplicative term into a scalar coefficient and the remaining
/// non-constant symbolic factor (if any).
fn strip_coefficient(expression: &Expr) -> (f64, Option<Expr>) {
    match expression {
        Expr::Constant(value) => (*value, None),
        Expr::Unary { operator: UnaryOperator::Negate, operand } => {
            let (coefficient, rest) = strip_coefficient(operand);
            (-coefficient, rest)
        }
        Expr::Binary { operator: BinaryOperator::Multiply, left, right } => {
            let (left_coefficient, left_rest) = strip_coefficient(left);
            let (right_coefficient, right_rest) = strip_coefficient(right);
            (left_coefficient * right_coefficient, combine(left_rest, right_rest))
        }
        other => (1.0, Some(other.clone())),
    }
}

fn combine(left: Option<Expr>, right: Option<Expr>) -> Option<Expr> {
    match (left, right) {
        (None, None) => None,
        (Some(expr), None) | (None, Some(expr)) => Some(expr),
        (Some(left), Some(right)) => Some(Expr::product(left, right)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    fn sym(name: &str) -> Expr {
        Expr::symbol(id(name))
    }

    #[test]
    fn describes_growth_and_decay_terms() {
        // dx/dt = 0.5*y - x
        let expression = Expr::difference(Expr::product(Expr::constant(0.5), sym("y")), sym("x"));
        let sentences = describe_law(&id("x"), &expression);
        assert_eq!(sentences.len(), 2);
        assert!(sentences[0].contains("increases in proportion to y (rate 0.5)"));
        assert!(sentences[1].contains("decreases in proportion to x"));
    }

    #[test]
    fn describes_constant_term() {
        let sentences = describe_law(&id("x"), &Expr::constant(-2.0));
        assert!(sentences[0].contains("loses a constant 2"));
    }
}
