use std::collections::BTreeSet;

use lawsynth_core::Identifier;
use lawsynth_data::Dataset;

use crate::{ContinuousProblem, DynamicsError};

/// A continuous identification problem with independently observed controls.
#[derive(Clone, Debug, PartialEq)]
pub struct ControlledProblem {
    continuous: ContinuousProblem,
    inputs: Vec<Identifier>,
}

impl ControlledProblem {
    pub fn new(
        dataset: Dataset,
        state: impl IntoIterator<Item = Identifier>,
        inputs: impl IntoIterator<Item = Identifier>,
    ) -> Result<Self, DynamicsError> {
        let inputs = inputs.into_iter().collect::<Vec<_>>();
        let continuous = ContinuousProblem::new(dataset, state)?;
        let state_set = continuous.state().iter().cloned().collect::<BTreeSet<_>>();
        let mut seen = BTreeSet::new();
        for input in &inputs {
            if !seen.insert(input.clone()) {
                return Err(DynamicsError::DuplicateVariable(input.to_string()));
            }
            if state_set.contains(input) {
                return Err(DynamicsError::StateInputOverlap(input.to_string()));
            }
            if !continuous.dataset().columns().contains_key(input) {
                return Err(DynamicsError::MissingInput(input.to_string()));
            }
        }
        Ok(Self { continuous, inputs })
    }

    pub fn continuous(&self) -> &ContinuousProblem {
        &self.continuous
    }

    pub fn inputs(&self) -> &[Identifier] {
        &self.inputs
    }
}

#[cfg(test)]
mod tests {
    use lawsynth_data::{NumericColumn, TimeAxis};

    use super::*;

    #[test]
    fn control_problem_rejects_state_input_overlap() {
        let x = Identifier::new("x").unwrap();
        let dataset = Dataset::new(
            TimeAxis::new(vec![0.0, 1.0]).unwrap(),
            [NumericColumn::new(x.clone(), vec![1.0, 2.0])],
        )
        .unwrap();
        assert_eq!(
            ControlledProblem::new(dataset, [x.clone()], [x]),
            Err(DynamicsError::StateInputOverlap("x".into()))
        );
    }
}
