use lawsynth_core::Identifier;
use lawsynth_data::Dataset;

use crate::{DynamicsConfig, DynamicsError, continuous::validate_with_config};

/// A validated data problem for fitting simultaneous discrete state recurrences.
#[derive(Clone, Debug, PartialEq)]
pub struct DiscreteProblem {
    dataset: Dataset,
    state: Vec<Identifier>,
}

impl DiscreteProblem {
    pub fn new(
        dataset: Dataset,
        state: impl IntoIterator<Item = Identifier>,
    ) -> Result<Self, DynamicsError> {
        Self::new_with_config(dataset, state, DynamicsConfig::default())
    }

    pub fn new_with_config(
        dataset: Dataset,
        state: impl IntoIterator<Item = Identifier>,
        config: DynamicsConfig,
    ) -> Result<Self, DynamicsError> {
        validate_with_config(&dataset, &state.into_iter().collect::<Vec<_>>(), config)
            .map(|state| Self { dataset, state })
    }

    pub fn dataset(&self) -> &Dataset {
        &self.dataset
    }

    pub fn state(&self) -> &[Identifier] {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;
    use lawsynth_data::{NumericColumn, TimeAxis};

    use super::*;

    #[test]
    fn validates_discrete_state_columns() {
        let x = Identifier::new("x").unwrap();
        let dataset = Dataset::new(
            TimeAxis::new(vec![0.0, 1.0]).unwrap(),
            [NumericColumn::new(x.clone(), vec![1.0, 2.0])],
        )
        .unwrap();
        assert_eq!(DiscreteProblem::new(dataset, [x.clone()]).unwrap().state(), &[x]);
    }
}
