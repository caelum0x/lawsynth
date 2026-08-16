use std::collections::{BTreeMap, btree_map::Entry};

use lawsynth_expr::Expr;

use crate::{Grammar, SymbolicConfig};

/// Enumerates unique scalar expressions in canonical order within strict bounds.
pub fn enumerate(grammar: &Grammar, config: &SymbolicConfig) -> Vec<Expr> {
    if config.max_candidates == 0 {
        return Vec::new();
    }
    let mut by_fingerprint = BTreeMap::new();
    let initial = std::iter::once(Expr::constant(1.0))
        .chain(grammar.terminals().iter().cloned().map(Expr::symbol));
    let mut frontier = initial
        .map(|expression| expression.simplify())
        .inspect(|expression| {
            by_fingerprint.insert(expression.to_canonical_string(), expression.clone());
        })
        .collect::<Vec<_>>();
    for _ in 1..=config.max_depth {
        let mut next = Vec::new();
        for left in &frontier {
            for right in by_fingerprint.values() {
                next.push(Expr::sum(left.clone(), right.clone()).simplify());
                if config.include_products {
                    next.push(Expr::product(left.clone(), right.clone()).simplify());
                }
            }
        }
        frontier.clear();
        for expression in next {
            if by_fingerprint.len() == config.max_candidates {
                break;
            }
            let key = expression.to_canonical_string();
            if let Entry::Vacant(entry) = by_fingerprint.entry(key) {
                entry.insert(expression.clone());
                frontier.push(expression);
            }
        }
        if frontier.is_empty() || by_fingerprint.len() == config.max_candidates {
            break;
        }
    }
    by_fingerprint.into_values().collect()
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;

    use super::*;

    #[test]
    fn enumeration_is_deterministic_and_includes_interactions() {
        let grammar =
            Grammar::scalar([Identifier::new("y").unwrap(), Identifier::new("x").unwrap()]);
        let config = SymbolicConfig {
            max_depth: 1,
            max_candidates: 20,
            include_products: true,
        };
        let candidates = enumerate(&grammar, &config);
        assert_eq!(candidates, enumerate(&grammar, &config));
        assert!(candidates.iter().any(|expression| {
            expression.to_canonical_string().contains("symbol:x")
                && expression.to_canonical_string().contains("symbol:y")
        }));
    }
}
