use crate::Expr;

/// Structural limits for accepting expressions from external callers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExpressionConfig {
    pub maximum_nodes: usize,
}

impl Default for ExpressionConfig {
    fn default() -> Self {
        Self {
            maximum_nodes: 1_024,
        }
    }
}

impl ExpressionConfig {
    pub fn accepts(self, expression: &Expr) -> bool {
        self.maximum_nodes > 0 && count_nodes(expression) <= self.maximum_nodes
    }
}

fn count_nodes(expression: &Expr) -> usize {
    match expression {
        Expr::Constant(_) | Expr::Symbol(_) => 1,
        Expr::Unary { operand, .. } => 1 + count_nodes(operand),
        Expr::Binary { left, right, .. } => 1 + count_nodes(left) + count_nodes(right),
    }
}
