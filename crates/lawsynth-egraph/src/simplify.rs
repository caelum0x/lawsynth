use lawsynth_core::Identifier;
use lawsynth_expr::Expr;

use crate::{
    RewriteConfig, RewriteError, RewriteLimits, RewriteSchedule, extract_lowest_cost, normalize,
};

/// Simplifies a single discovered-law expression tree into its cost-minimal
/// equivalent form under bounded, deterministic equality saturation.
///
/// The returned expression is value-preserving on the input's domain (see the
/// per-rule documentation in [`crate::RewriteRule`] and the soundness contract
/// in `specs/egraph-simplification/README.md`) and is chosen to minimize the
/// scalar node count, with ties broken by canonical representation. Identical
/// inputs yield bit-identical output.
///
/// # Errors
///
/// Returns [`RewriteError::InvalidConfig`] when the configured pass bound is
/// zero, and [`RewriteError::LimitExceeded`] when the input exceeds the default
/// structural node limit (which keeps saturation bounded and terminating).
pub fn simplify_expr(expression: &Expr, config: &RewriteConfig) -> Result<Expr, RewriteError> {
    // Validate the schedule (rejects a zero pass bound) and enforce the hard
    // node ceiling so saturation is always bounded and terminating.
    let schedule = RewriteSchedule::from_config(config)?;
    RewriteLimits::default().check(expression)?;
    Ok(saturate_to_minimal_cost(expression.clone(), schedule.passes))
}

/// Simplifies every field of a discovered law, preserving the caller's field
/// order. Intended for cleaning up a law's expression trees before display or
/// export.
///
/// # Errors
///
/// Propagates the first [`RewriteError`] produced while simplifying any field.
pub fn simplify_law(
    fields: &[(Identifier, Expr)],
    config: &RewriteConfig,
) -> Result<Vec<(Identifier, Expr)>, RewriteError> {
    fields
        .iter()
        .map(|(name, expression)| Ok((name.clone(), simplify_expr(expression, config)?)))
        .collect()
}

/// Runs bounded saturation, collecting every distinct intermediate form, then
/// extracts the cost-minimal member. Extraction over the collected forms mirrors
/// an e-graph's "pick the cheapest equivalent term" step while remaining fully
/// deterministic.
fn saturate_to_minimal_cost(expression: Expr, passes: usize) -> Expr {
    let mut candidates = vec![expression.clone()];
    let mut current = expression;
    for _ in 0..passes {
        let next = normalize(current.clone());
        if next == current {
            break;
        }
        candidates.push(next.clone());
        current = next;
    }
    // `candidates` is always non-empty, so extraction cannot return `None`.
    extract_lowest_cost(&candidates).unwrap_or(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression_cost;
    use lawsynth_expr::{BinaryOperator, UnaryOperator, parse};

    fn identifier(name: &str) -> Identifier {
        Identifier::new(name).unwrap()
    }

    #[test]
    fn simplifies_a_messy_expression_to_a_compact_form() {
        let expression = parse("0 + (x * 1) + (y - y)").unwrap();
        let simplified = simplify_expr(&expression, &RewriteConfig::default()).unwrap();
        assert_eq!(simplified, Expr::symbol(identifier("x")));
    }

    #[test]
    fn leaves_an_already_minimal_expression_unchanged() {
        let expression = parse("x + y").unwrap();
        let normalized = normalize(expression.clone());
        let simplified = simplify_expr(&normalized, &RewriteConfig::default()).unwrap();
        assert_eq!(simplified, normalized);
    }

    #[test]
    fn never_increases_cost() {
        let expression = parse("a*b + a*c + 0 * z").unwrap();
        let simplified = simplify_expr(&expression, &RewriteConfig::default()).unwrap();
        assert!(expression_cost(&simplified) <= expression_cost(&expression));
    }

    #[test]
    fn rejects_a_zero_pass_schedule() {
        let expression = Expr::symbol(identifier("x"));
        assert_eq!(
            simplify_expr(&expression, &RewriteConfig { max_passes: 0 }),
            Err(RewriteError::InvalidConfig)
        );
    }

    #[test]
    fn rejects_expressions_beyond_the_node_limit() {
        let mut expression = Expr::symbol(identifier("x"));
        for _ in 0..512 {
            expression = Expr::unary(UnaryOperator::Negate, expression);
        }
        assert_eq!(
            simplify_expr(&expression, &RewriteConfig::default()),
            Err(RewriteError::LimitExceeded)
        );
    }

    #[test]
    fn simplify_law_preserves_field_order() {
        let fields = vec![
            (identifier("dx"), parse("x * 1").unwrap()),
            (identifier("dy"), parse("y + 0").unwrap()),
        ];
        let simplified = simplify_law(&fields, &RewriteConfig::default()).unwrap();
        assert_eq!(simplified[0].0, identifier("dx"));
        assert_eq!(simplified[0].1, Expr::symbol(identifier("x")));
        assert_eq!(simplified[1].0, identifier("dy"));
        assert_eq!(simplified[1].1, Expr::symbol(identifier("y")));
    }

    #[test]
    fn simplification_is_bit_identical_across_runs() {
        let expression = parse("(a*b + a*c) * 1 + 0 - log(exp(d))").unwrap();
        let first = simplify_expr(&expression, &RewriteConfig::default()).unwrap();
        let second = simplify_expr(&expression, &RewriteConfig::default()).unwrap();
        assert_eq!(first.to_canonical_string(), second.to_canonical_string());
    }

    #[test]
    fn terminates_on_a_branchy_input() {
        // A deep, wide tree of identities that must all collapse within limits.
        // Each iteration adds 8 nodes, so 20 stays under the 256-node ceiling.
        let mut expression = Expr::constant(0.0);
        for index in 0..20 {
            let symbol = Expr::symbol(identifier(&format!("s{index}")));
            let term = Expr::product(symbol, Expr::constant(1.0));
            expression = Expr::sum(expression, Expr::difference(term.clone(), term));
        }
        let simplified = simplify_expr(&expression, &RewriteConfig::default()).unwrap();
        assert_eq!(simplified, Expr::constant(0.0));
    }

    #[test]
    fn extraction_selects_the_cheapest_power_form() {
        let expression = Expr::binary(
            BinaryOperator::Power,
            Expr::binary(BinaryOperator::Power, Expr::symbol(identifier("x")), Expr::constant(2.0)),
            Expr::constant(3.0),
        );
        let simplified = simplify_expr(&expression, &RewriteConfig::default()).unwrap();
        assert_eq!(
            simplified,
            Expr::binary(BinaryOperator::Power, Expr::symbol(identifier("x")), Expr::constant(6.0))
        );
    }
}
