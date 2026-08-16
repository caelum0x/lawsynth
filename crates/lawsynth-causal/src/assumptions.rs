use crate::{CausalError, CausalGraph, Result};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CausalAssumption {
    NoUnmeasuredConfounding { cause: String, effect: String },
    Faithfulness,
    CausalSufficiency,
}
#[derive(Debug, Clone, Default)]
pub struct AssumptionSet {
    assumptions: BTreeSet<CausalAssumption>,
}
impl AssumptionSet {
    pub fn declare(&mut self, assumption: CausalAssumption) {
        self.assumptions.insert(assumption);
    }
    pub fn contains(&self, assumption: &CausalAssumption) -> bool {
        self.assumptions.contains(assumption)
    }
    pub fn validate_against(&self, graph: &CausalGraph) -> Result<()> {
        for a in &self.assumptions {
            if let CausalAssumption::NoUnmeasuredConfounding { cause, effect } = a {
                if !graph.has_edge(cause, effect) {
                    return Err(CausalError::InvalidParameter(
                        "confounding assumption must name a graph edge",
                    ));
                }
            }
        }
        Ok(())
    }
    pub fn iter(&self) -> impl Iterator<Item = &CausalAssumption> {
        self.assumptions.iter()
    }
}
