use lawsynth_expr::Expr;
use std::collections::BTreeMap;

/// A deterministic, structurally unique set of symbolic candidates.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Population {
    candidates: BTreeMap<String, Expr>,
}
impl Population {
    pub fn new(candidates: impl IntoIterator<Item = Expr>) -> Self {
        let mut population = Self::default();
        population.extend(candidates);
        population
    }
    pub fn extend(&mut self, candidates: impl IntoIterator<Item = Expr>) {
        for candidate in candidates {
            let candidate = candidate.simplify();
            self.candidates
                .entry(candidate.to_canonical_string())
                .or_insert(candidate);
        }
    }
    pub fn expressions(&self) -> impl Iterator<Item = &Expr> {
        self.candidates.values()
    }
    pub fn len(&self) -> usize {
        self.candidates.len()
    }
    pub fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }
}
