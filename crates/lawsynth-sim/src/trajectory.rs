use std::collections::BTreeMap;

use lawsynth_core::Identifier;

/// A sampled state trajectory in deterministic state-id order.
#[derive(Clone, Debug, PartialEq)]
pub struct Trajectory {
    pub time: Vec<f64>,
    pub values: BTreeMap<Identifier, Vec<f64>>,
}

impl Trajectory {
    pub(crate) fn from_initial(time: f64, initial_state: &BTreeMap<Identifier, f64>) -> Self {
        Self {
            time: vec![time],
            values: initial_state
                .iter()
                .map(|(id, value)| (id.clone(), vec![*value]))
                .collect(),
        }
    }

    pub(crate) fn push(&mut self, time: f64, state: &BTreeMap<Identifier, f64>) {
        self.time.push(time);
        for (id, value) in state {
            self.values
                .get_mut(id)
                .expect("trajectory state shape is fixed")
                .push(*value);
        }
    }

    pub fn samples(&self) -> usize {
        self.time.len()
    }
}
