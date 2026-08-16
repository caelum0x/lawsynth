use std::collections::BTreeMap;

use lawsynth_expr::Expr;

use crate::{RewriteConfig, normalize};

/// One deterministic equivalence class keyed by its extracted canonical form.
#[derive(Clone, Debug, PartialEq)]
pub struct EquivalenceClass {
    pub canonical: Expr,
    pub members: Vec<Expr>,
}

/// A small scalar e-graph using local algebraic rewrites as its safe rule set.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EquivalenceGraph {
    classes: BTreeMap<String, EquivalenceClass>,
}

impl EquivalenceGraph {
    pub fn add(&mut self, expression: Expr, config: &RewriteConfig) -> &EquivalenceClass {
        let canonical = saturate(expression.clone(), config);
        let key = canonical.to_canonical_string();
        let class = self.classes.entry(key).or_insert_with(|| EquivalenceClass {
            canonical,
            members: Vec::new(),
        });
        if !class.members.contains(&expression) {
            class.members.push(expression);
        }
        class
    }

    pub fn classes(&self) -> impl Iterator<Item = &EquivalenceClass> {
        self.classes.values()
    }

    pub fn equivalent(&self, left: &Expr, right: &Expr, config: &RewriteConfig) -> bool {
        saturate(left.clone(), config).to_canonical_string()
            == saturate(right.clone(), config).to_canonical_string()
    }
}

fn saturate(mut expression: Expr, config: &RewriteConfig) -> Expr {
    for _ in 0..config.max_passes {
        let rewritten = normalize(expression.clone());
        if rewritten == expression {
            break;
        }
        expression = rewritten;
    }
    expression
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;

    use super::*;

    #[test]
    fn merges_locally_equivalent_expressions() {
        let x = Expr::symbol(Identifier::new("x").unwrap());
        let zero = Expr::constant(0.0);
        let mut graph = EquivalenceGraph::default();
        graph.add(Expr::sum(x.clone(), zero), &RewriteConfig::default());
        graph.add(x.clone(), &RewriteConfig::default());
        assert_eq!(graph.classes().count(), 1);
        assert!(!graph.equivalent(
            &x,
            &Expr::sum(Expr::constant(0.0), Expr::constant(1.0)),
            &RewriteConfig { max_passes: 0 }
        ));
    }
}
