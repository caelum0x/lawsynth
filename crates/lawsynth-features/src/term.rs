use lawsynth_expr::Expr;

/// A named expression column in a design matrix.
#[derive(Clone, Debug, PartialEq)]
pub struct FeatureTerm {
    pub name: String,
    pub expression: Expr,
}
