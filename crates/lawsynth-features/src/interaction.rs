use std::collections::BTreeSet;

use lawsynth_core::Identifier;
use lawsynth_expr::{Expr, print};

use crate::{FeatureError, FeatureTerm};

/// Creates one product for every unordered pair of distinct variables.
pub(crate) fn terms(variables: &[Identifier]) -> Result<Vec<FeatureTerm>, FeatureError> {
    let mut seen = BTreeSet::new();
    for variable in variables {
        if !seen.insert(variable.clone()) {
            return Err(FeatureError::DuplicateVariable(variable.to_string()));
        }
    }

    Ok(variables
        .iter()
        .enumerate()
        .flat_map(|(left_index, left)| {
            variables[left_index + 1..].iter().map(move |right| {
                let expression =
                    Expr::product(Expr::symbol(left.clone()), Expr::symbol(right.clone()));
                FeatureTerm {
                    name: print(&expression),
                    expression,
                }
            })
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;

    use super::*;

    #[test]
    fn rejects_repeated_variables_instead_of_emitting_duplicate_features() {
        let x = Identifier::new("x").unwrap();
        assert_eq!(
            terms(&[x.clone(), x]),
            Err(FeatureError::DuplicateVariable("x".into()))
        );
    }
}
