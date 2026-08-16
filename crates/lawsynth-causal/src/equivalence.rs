use crate::CausalGraph;
use std::collections::BTreeSet;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkovEquivalence {
    pub skeleton: BTreeSet<(String, String)>,
    pub colliders: BTreeSet<(String, String, String)>,
}
pub fn equivalence_class(graph: &CausalGraph) -> MarkovEquivalence {
    let skeleton = graph
        .edges()
        .map(|e| {
            if e.from < e.to {
                (e.from.clone(), e.to.clone())
            } else {
                (e.to.clone(), e.from.clone())
            }
        })
        .collect();
    let mut colliders = BTreeSet::new();
    for middle in graph.variables() {
        let parents = graph.parents(middle).expect("graph variable");
        for i in 0..parents.len() {
            for j in i + 1..parents.len() {
                if !graph.has_edge(parents[i], parents[j])
                    && !graph.has_edge(parents[j], parents[i])
                {
                    colliders.insert((parents[i].into(), middle.into(), parents[j].into()));
                }
            }
        }
    }
    MarkovEquivalence {
        skeleton,
        colliders,
    }
}
