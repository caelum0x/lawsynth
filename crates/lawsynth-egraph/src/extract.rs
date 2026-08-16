use crate::expression_cost;
use lawsynth_expr::Expr;

/// Chooses the lowest-cost member, breaking ties by canonical representation.
pub fn extract_lowest_cost(expressions: &[Expr]) -> Option<Expr> {
    expressions
        .iter()
        .min_by(|left, right| {
            expression_cost(left)
                .cmp(&expression_cost(right))
                .then_with(|| left.to_canonical_string().cmp(&right.to_canonical_string()))
        })
        .cloned()
}
