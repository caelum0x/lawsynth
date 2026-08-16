use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use std::collections::BTreeSet;

/// Structural facts that can be computed without evaluating an expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpressionAnalysis {
    pub nodes: usize,
    pub symbols: BTreeSet<Identifier>,
}

impl ExpressionAnalysis {
    pub fn inspect(expression: &Expr) -> Self {
        let mut symbols = BTreeSet::new();
        let nodes = inspect(expression, &mut symbols);
        Self { nodes, symbols }
    }
}
fn inspect(expression: &Expr, symbols: &mut BTreeSet<Identifier>) -> usize {
    match expression {
        Expr::Constant(_) => 1,
        Expr::Symbol(symbol) => {
            symbols.insert(symbol.clone());
            1
        }
        Expr::Unary { operand, .. } => 1 + inspect(operand, symbols),
        Expr::Binary { left, right, .. } => 1 + inspect(left, symbols) + inspect(right, symbols),
    }
}
