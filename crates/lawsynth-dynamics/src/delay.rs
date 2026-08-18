use lawsynth_core::Identifier;
use lawsynth_data::Dataset;

use crate::{DynamicsError, continuous::validate};

/// A delayed-state identification problem with an integer observation lag.
#[derive(Clone, Debug, PartialEq)]
pub struct DelayedProblem {
    dataset: Dataset,
    state: Vec<Identifier>,
    lag: usize,
}

/// Aligned state observations, where row `i` pairs a present state with its
/// state observed `lag` samples earlier.
#[derive(Clone, Debug, PartialEq)]
pub struct DelaySamples {
    pub start_index: usize,
    pub current: Vec<Vec<f64>>,
    pub lagged: Vec<Vec<f64>>,
}

impl DelayedProblem {
    pub fn new(
        dataset: Dataset,
        state: impl IntoIterator<Item = Identifier>,
        lag: usize,
    ) -> Result<Self, DynamicsError> {
        let state = validate(&dataset, &state.into_iter().collect::<Vec<_>>())?;
        if lag == 0 || lag >= dataset.time().len() {
            return Err(DynamicsError::InvalidLag);
        }
        Ok(Self { dataset, state, lag })
    }

    pub fn lag(&self) -> usize {
        self.lag
    }

    pub fn samples(&self) -> DelaySamples {
        let current = (self.lag..self.dataset.time().len())
            .map(|row| self.state.iter().map(|id| self.dataset.columns()[id].values[row]).collect())
            .collect();
        let lagged = (self.lag..self.dataset.time().len())
            .map(|row| {
                self.state
                    .iter()
                    .map(|id| self.dataset.columns()[id].values[row - self.lag])
                    .collect()
            })
            .collect();
        DelaySamples { start_index: self.lag, current, lagged }
    }
}

#[cfg(test)]
mod tests {
    use lawsynth_data::{NumericColumn, TimeAxis};

    use super::*;

    #[test]
    fn creates_only_observationally_supported_delay_rows() {
        let x = Identifier::new("x").unwrap();
        let dataset = Dataset::new(
            TimeAxis::new(vec![0.0, 1.0, 2.0]).unwrap(),
            [NumericColumn::new(x.clone(), vec![1.0, 2.0, 4.0])],
        )
        .unwrap();
        assert_eq!(
            DelayedProblem::new(dataset, [x], 1).unwrap().samples().lagged,
            vec![vec![1.0], vec![2.0]]
        );
    }
}
