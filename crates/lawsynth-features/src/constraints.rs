use std::collections::BTreeSet;

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;

use crate::FeatureTerm;

/// Structural inclusion rules for feature-library terms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeatureConstraint {
    /// Retain only terms whose every symbol belongs to this allow-list.
    AllowedSymbols(BTreeSet<Identifier>),
    /// Retain terms containing no more than this many AST nodes.
    MaximumNodes(usize),
    /// Exclude constant-only terms such as the polynomial intercept.
    RequireSymbol,
}

pub(crate) fn allows(rule: &FeatureConstraint, term: &FeatureTerm) -> bool {
    match rule {
        FeatureConstraint::AllowedSymbols(symbols) => {
            expression_symbols(&term.expression).is_subset(symbols)
        }
        FeatureConstraint::MaximumNodes(maximum) => expression_nodes(&term.expression) <= *maximum,
        FeatureConstraint::RequireSymbol => !expression_symbols(&term.expression).is_empty(),
    }
}

fn expression_nodes(expression: &Expr) -> usize {
    match expression {
        Expr::Constant(_) | Expr::Symbol(_) => 1,
        Expr::Unary { operand, .. } => 1 + expression_nodes(operand),
        Expr::Binary { left, right, .. } => 1 + expression_nodes(left) + expression_nodes(right),
    }
}

fn expression_symbols(expression: &Expr) -> BTreeSet<Identifier> {
    let mut symbols = BTreeSet::new();
    collect_symbols(expression, &mut symbols);
    symbols
}

fn collect_symbols(expression: &Expr, symbols: &mut BTreeSet<Identifier>) {
    match expression {
        Expr::Constant(_) => {}
        Expr::Symbol(symbol) => {
            symbols.insert(symbol.clone());
        }
        Expr::Unary { operand, .. } => collect_symbols(operand, symbols),
        Expr::Binary { left, right, .. } => {
            collect_symbols(left, symbols);
            collect_symbols(right, symbols);
        }
    }
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;
    use lawsynth_expr::Expr;

    use super::*;

    #[test]
    fn counts_nodes_and_discovers_nested_symbols() {
        let x = Identifier::new("x").unwrap();
        let y = Identifier::new("y").unwrap();
        let expression = Expr::product(Expr::symbol(x.clone()), Expr::symbol(y.clone()));
        let term = FeatureTerm {
            name: "x * y".into(),
            expression,
        };
        assert!(allows(&FeatureConstraint::MaximumNodes(3), &term));
        assert!(!allows(
            &FeatureConstraint::AllowedSymbols([x].into_iter().collect()),
            &term
        ));
        assert!(allows(&FeatureConstraint::RequireSymbol, &term));
    }
}
