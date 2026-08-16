use lawsynth_core::Identifier;
use lawsynth_data::Dataset;

use crate::{DynamicsError, continuous::validate};

/// A validated multivariate dataset for discovering implicit relations F(x)=0.
#[derive(Clone, Debug, PartialEq)]
pub struct ImplicitProblem {
    dataset: Dataset,
    variables: Vec<Identifier>,
}

impl ImplicitProblem {
    pub fn new(
        dataset: Dataset,
        variables: impl IntoIterator<Item = Identifier>,
    ) -> Result<Self, DynamicsError> {
        let variables = validate(&dataset, &variables.into_iter().collect::<Vec<_>>())?;
        Ok(Self { dataset, variables })
    }

    pub fn dataset(&self) -> &Dataset {
        &self.dataset
    }

    pub fn variables(&self) -> &[Identifier] {
        &self.variables
    }
}
