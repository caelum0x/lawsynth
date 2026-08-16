use lawsynth_core::Identifier;
use lawsynth_expr::Expr;

/// A continuous law whose target derivative equals its expression.
#[derive(Clone, Debug, PartialEq)]
pub struct ContinuousLaw {
    pub target: Identifier,
    pub expression: Expr,
}

impl ContinuousLaw {
    pub fn new(target: Identifier, expression: Expr) -> Self {
        Self { target, expression }
    }
}

/// A discrete law whose next target value equals its expression.
#[derive(Clone, Debug, PartialEq)]
pub struct DiscreteLaw {
    pub target: Identifier,
    pub expression: Expr,
}

impl DiscreteLaw {
    pub fn new(target: Identifier, expression: Expr) -> Self {
        Self { target, expression }
    }
}
