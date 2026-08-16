use std::collections::BTreeMap;

use lawsynth_core::Identifier;
use lawsynth_expr::Environment;

/// Immutable values visible while one law evaluation is performed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SimulationContext {
    pub state: BTreeMap<Identifier, f64>,
    pub parameters: BTreeMap<Identifier, f64>,
    pub inputs: BTreeMap<Identifier, f64>,
}

impl SimulationContext {
    pub fn new(
        state: BTreeMap<Identifier, f64>,
        parameters: BTreeMap<Identifier, f64>,
        inputs: BTreeMap<Identifier, f64>,
    ) -> Self {
        Self {
            state,
            parameters,
            inputs,
        }
    }

    /// Builds the canonical expression environment, with later namespaces only
    /// present when their identifiers were explicitly declared non-conflicting.
    pub fn environment(&self) -> Environment {
        self.state
            .iter()
            .chain(&self.parameters)
            .chain(&self.inputs)
            .map(|(id, value)| (id.clone(), *value))
            .collect()
    }
}
