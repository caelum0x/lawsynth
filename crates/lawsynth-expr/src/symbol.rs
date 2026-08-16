use crate::Expr;
use lawsynth_core::Identifier;
use std::collections::BTreeSet;

/// Collects each symbol used by an expression in deterministic lexical order.
pub fn symbols(expression: &Expr) -> BTreeSet<Identifier> {
    let mut result = BTreeSet::new();
    collect(expression, &mut result);
    result
}
fn collect(expression: &Expr, symbols: &mut BTreeSet<Identifier>) {
    match expression {
        Expr::Constant(_) => {}
        Expr::Symbol(identifier) => {
            symbols.insert(identifier.clone());
        }
        Expr::Unary { operand, .. } => collect(operand, symbols),
        Expr::Binary { left, right, .. } => {
            collect(left, symbols);
            collect(right, symbols);
        }
    }
}
