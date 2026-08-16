use crate::{RewriteError, expression_cost};
use lawsynth_expr::Expr;

/// Hard structural bounds applied before rewrite work is scheduled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RewriteLimits {
    pub maximum_nodes: usize,
}
impl Default for RewriteLimits {
    fn default() -> Self {
        Self { maximum_nodes: 256 }
    }
}
impl RewriteLimits {
    pub fn check(self, expression: &Expr) -> Result<(), RewriteError> {
        if self.maximum_nodes == 0 {
            return Err(RewriteError::InvalidConfig);
        }
        if expression_cost(expression) > self.maximum_nodes {
            Err(RewriteError::LimitExceeded)
        } else {
            Ok(())
        }
    }
}
