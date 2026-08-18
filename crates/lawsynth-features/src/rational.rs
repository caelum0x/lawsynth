use lawsynth_core::Identifier;
use lawsynth_expr::{Expr, print};

use crate::FeatureTerm;

/// Returns protected, bounded rational features for each scalar variable.
pub(crate) fn bounded_terms(variables: &[Identifier]) -> Vec<FeatureTerm> {
    variables
        .iter()
        .map(|variable| {
            let numerator = Expr::symbol(variable.clone());
            let denominator = Expr::sum(
                Expr::constant(1.0),
                Expr::product(Expr::symbol(variable.clone()), Expr::symbol(variable.clone())),
            );
            let expression = Expr::quotient(numerator, denominator).simplify();
            FeatureTerm { name: print(&expression), expression }
        })
        .collect()
}
