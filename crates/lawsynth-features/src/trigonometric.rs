use lawsynth_core::Identifier;
use lawsynth_expr::{Expr, UnaryOperator, print};

use crate::FeatureTerm;

pub(crate) fn terms(variables: &[Identifier]) -> Vec<FeatureTerm> {
    variables
        .iter()
        .flat_map(|variable| {
            [UnaryOperator::Sin, UnaryOperator::Cos]
                .into_iter()
                .map(|operator| {
                    let expression = Expr::unary(operator, Expr::symbol(variable.clone()));
                    FeatureTerm {
                        name: print(&expression),
                        expression,
                    }
                })
        })
        .collect()
}
