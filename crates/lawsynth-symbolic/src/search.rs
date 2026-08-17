use std::collections::{BTreeMap, btree_map::Entry};

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_units::{Dimension, admits_scaled_dimension};

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

/// Enumerates candidate expressions, retaining only those dimensionally
/// consistent with `target` under the per-symbol `dimensions`.
///
/// Each surviving expression is fitted with a free affine calibration downstream
/// (a wildcard scale and offset), so admissibility asks whether *some* dimensionful
/// coefficient could rescale the expression to the target derivative's dimension —
/// exactly [`admits_scaled_dimension`]. With an empty `dimensions` map every
/// candidate is a dimensional wildcard, so the result is identical to
/// [`enumerate`]: pruning never changes results when units are absent.
pub fn enumerate_admissible(
    grammar: &Grammar,
    config: &SymbolicConfig,
    dimensions: &BTreeMap<Identifier, Dimension>,
    target: Dimension,
) -> Vec<Expr> {
    enumerate(grammar, config)
        .into_iter()
        .filter(|expression| admits_scaled_dimension(expression, dimensions, target))
        .collect()
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;
    use lawsynth_units::Unit;

    use super::*;

    #[test]
    fn enumeration_is_deterministic_and_includes_interactions() {
        let grammar =
            Grammar::scalar([Identifier::new("y").unwrap(), Identifier::new("x").unwrap()]);
        let config = SymbolicConfig { max_depth: 1, max_candidates: 20, include_products: true };
        let candidates = enumerate(&grammar, &config);
        assert_eq!(candidates, enumerate(&grammar, &config));
        assert!(candidates.iter().any(|expression| {
            expression.to_canonical_string().contains("symbol:x")
                && expression.to_canonical_string().contains("symbol:y")
        }));
    }

    #[test]
    fn admissible_enumeration_matches_plain_enumeration_without_units() {
        let grammar =
            Grammar::scalar([Identifier::new("x").unwrap(), Identifier::new("v").unwrap()]);
        let config = SymbolicConfig { max_depth: 1, max_candidates: 24, include_products: true };
        let plain = enumerate(&grammar, &config);
        let admissible = enumerate_admissible(
            &grammar,
            &config,
            &BTreeMap::new(),
            Unit::parse("m/s^2").unwrap().dimension(),
        );
        assert_eq!(plain, admissible);
    }

    #[test]
    fn admissible_enumeration_drops_dimensionally_impossible_sums() {
        let x = Identifier::new("x").unwrap();
        let v = Identifier::new("v").unwrap();
        let grammar = Grammar::scalar([x.clone(), v.clone()]);
        let config = SymbolicConfig { max_depth: 1, max_candidates: 32, include_products: true };
        let dimensions = BTreeMap::from([
            (x, Unit::parse("m").unwrap().dimension()),
            (v, Unit::parse("m/s").unwrap().dimension()),
        ]);
        let target = Unit::parse("m/s^2").unwrap().dimension();
        let admissible = enumerate_admissible(&grammar, &config, &dimensions, target);
        // A length-plus-velocity sum is dimensionally impossible and must be gone,
        // while the bare velocity survives (a coefficient rescales it to m/s²).
        assert!(admissible.iter().all(|expression| {
            let canonical = expression.to_canonical_string();
            !(canonical == "binary:Add(symbol:v,symbol:x)"
                || canonical == "binary:Add(symbol:x,symbol:v)")
        }));
        assert!(admissible.iter().any(|expression| expression.to_canonical_string() == "symbol:v"));
        assert!(admissible.len() < enumerate(&grammar, &config).len());
    }
}
