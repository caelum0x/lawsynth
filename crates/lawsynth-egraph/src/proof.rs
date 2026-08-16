use crate::{RewriteRule, normalize};
use lawsynth_expr::Expr;

/// A replayable statement that a local rewrite normalized one expression to another.
#[derive(Clone, Debug, PartialEq)]
pub struct RewriteProof {
    pub rule: RewriteRule,
    pub before: Expr,
    pub after: Expr,
}
impl RewriteProof {
    pub fn verify(&self) -> bool {
        self.after == normalize(self.before.clone())
    }
}
