use std::collections::BTreeSet;

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;

/// Returns all symbols read by an expression in stable lexical order.
pub fn expression_symbols(expression: &Expr) -> BTreeSet<Identifier> {
    let mut symbols = BTreeSet::new();
    collect(expression, &mut symbols);
    symbols
}

fn collect(expression: &Expr, symbols: &mut BTreeSet<Identifier>) {
    match expression {
        Expr::Constant(_) => {}
        Expr::Symbol(id) => {
            symbols.insert(id.clone());
        }
        Expr::Unary { operand, .. } => collect(operand, symbols),
        Expr::Binary { left, right, .. } => {
            collect(left, symbols);
            collect(right, symbols);
        }
    }
}

#[cfg(test)]
mod tests {
    use lawsynth_expr::Expr;

    use super::*;

    #[test]
    fn lists_unique_symbols_in_order() {
        let id = |value| Identifier::new(value).unwrap();
        let expression = Expr::sum(
            Expr::symbol(id("z")),
            Expr::product(Expr::symbol(id("x")), Expr::symbol(id("z"))),
        );
        assert_eq!(
            expression_symbols(&expression)
                .into_iter()
                .collect::<Vec<_>>(),
            vec![id("x"), id("z")]
        );
    }
}
