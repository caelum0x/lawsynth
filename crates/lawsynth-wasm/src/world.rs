use crate::{Expression, WasmError};
use std::collections::{BTreeMap, BTreeSet};

/// Deterministic continuous-time model supported by the portable surface.
#[derive(Clone, Debug, PartialEq)]
pub struct World {
    pub variables: Vec<String>,
    pub initial_state: Vec<f64>,
    pub derivatives: Vec<Expression>,
}
impl World {
    pub fn new(
        variables: Vec<String>,
        initial_state: Vec<f64>,
        derivatives: Vec<Expression>,
    ) -> Result<Self, WasmError> {
        if variables.is_empty()
            || variables.len() != initial_state.len()
            || variables.len() != derivatives.len()
        {
            return Err(WasmError::InvalidWorld(
                "variables, initial_state, and derivatives must have matching nonzero lengths"
                    .into(),
            ));
        }
        let mut names = BTreeSet::new();
        for name in &variables {
            if !valid_name(name) || !names.insert(name) {
                return Err(WasmError::InvalidWorld(format!(
                    "invalid or duplicate variable {name}"
                )));
            }
        }
        if initial_state.iter().any(|value| !value.is_finite()) {
            return Err(WasmError::InvalidWorld("initial state must be finite".into()));
        }
        Ok(Self { variables, initial_state, derivatives })
    }
    pub fn state_map(&self, time: f64, state: &[f64]) -> Result<BTreeMap<String, f64>, WasmError> {
        if state.len() != self.variables.len()
            || !time.is_finite()
            || state.iter().any(|value| !value.is_finite())
        {
            return Err(WasmError::InvalidWorld("invalid state vector".into()));
        }
        let mut values = BTreeMap::new();
        values.insert("t".into(), time);
        for (name, value) in self.variables.iter().zip(state) {
            values.insert(name.clone(), *value);
        }
        Ok(values)
    }
    pub fn derivative_at(&self, time: f64, state: &[f64]) -> Result<Vec<f64>, WasmError> {
        let values = self.state_map(time, state)?;
        self.derivatives.iter().map(|expression| expression.evaluate(&values)).collect()
    }
}
fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && name != "t"
}
