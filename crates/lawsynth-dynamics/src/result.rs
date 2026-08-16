use lawsynth_core::Identifier;

use crate::{DiscreteProblem, DynamicsError};

/// One-step state transitions extracted without changing the sample order.
#[derive(Clone, Debug, PartialEq)]
pub struct DiscreteTransitions {
    pub times: Vec<f64>,
    pub current: Vec<Vec<f64>>,
    pub next: Vec<Vec<f64>>,
}

pub fn discrete_transitions(
    problem: &DiscreteProblem,
) -> Result<DiscreteTransitions, DynamicsError> {
    let dataset = problem.dataset();
    let state: &[Identifier] = problem.state();
    if dataset.time().len() < 2 {
        return Err(DynamicsError::TooFewSamples);
    }
    let current = (0..dataset.time().len() - 1)
        .map(|row| {
            state
                .iter()
                .map(|id| dataset.columns()[id].values[row])
                .collect()
        })
        .collect();
    let next = (1..dataset.time().len())
        .map(|row| {
            state
                .iter()
                .map(|id| dataset.columns()[id].values[row])
                .collect()
        })
        .collect();
    Ok(DiscreteTransitions {
        times: dataset.time().values()[..dataset.time().len() - 1].to_vec(),
        current,
        next,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use lawsynth_core::Identifier;
    use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
    #[test]
    fn preserves_one_step_transition_order() {
        let x = Identifier::new("x").unwrap();
        let problem = DiscreteProblem::new(
            Dataset::new(
                TimeAxis::new(vec![0.0, 1.0, 2.0]).unwrap(),
                [NumericColumn::new(x.clone(), vec![1.0, 3.0, 9.0])],
            )
            .unwrap(),
            [x],
        )
        .unwrap();
        assert_eq!(
            discrete_transitions(&problem).unwrap().next,
            vec![vec![3.0], vec![9.0]]
        );
    }
}
