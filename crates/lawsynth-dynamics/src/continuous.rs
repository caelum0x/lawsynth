use lawsynth_core::Identifier;
use lawsynth_data::Dataset;

use crate::{DynamicsConfig, DynamicsError};

/// A validated data problem for fitting differential state-transition laws.
#[derive(Clone, Debug, PartialEq)]
pub struct ContinuousProblem {
    dataset: Dataset,
    state: Vec<Identifier>,
}

impl ContinuousProblem {
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

pub(crate) fn validate(
    dataset: &Dataset,
    state: &[Identifier],
) -> Result<Vec<Identifier>, DynamicsError> {
    validate_with_config(dataset, state, DynamicsConfig::default())
}

pub(crate) fn validate_with_config(
    dataset: &Dataset,
    state: &[Identifier],
    config: DynamicsConfig,
) -> Result<Vec<Identifier>, DynamicsError> {
    config.validate()?;
    if dataset.time().len() < config.minimum_samples {
        return Err(DynamicsError::TooFewSamples);
    }
    if state.is_empty() {
        return Err(DynamicsError::NoStates);
    }
    let mut seen = std::collections::BTreeSet::new();
    for id in state {
        if !seen.insert(id.clone()) {
            return Err(DynamicsError::DuplicateVariable(id.to_string()));
        }
        if !dataset.columns().contains_key(id) {
            return Err(DynamicsError::MissingState(id.to_string()));
        }
    }
    Ok(state.to_vec())
}
