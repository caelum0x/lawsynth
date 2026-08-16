use std::collections::{BTreeMap, BTreeSet};

use lawsynth_core::Identifier;
use lawsynth_world::{DiscreteWorld, VariableRole};

use crate::{
    CompiledDiscreteWorld, DiscreteSimulationConfig, SimulationContext, SimulationError,
    SimulationRequest, Trajectory, evaluate_discrete,
};

/// Simulates simultaneous discrete state recurrences.
pub fn simulate_discrete(
    world: &DiscreteWorld,
    config: DiscreteSimulationConfig,
    request: &SimulationRequest,
) -> Result<Trajectory, SimulationError> {
    let states: BTreeSet<_> = world.state_ids().cloned().collect();
    for id in &states {
        let value = request
            .initial_state
            .get(id)
            .ok_or_else(|| SimulationError::MissingInitialState(id.clone()))?;
        ensure_finite(id, *value)?;
    }
    for (id, value) in &request.initial_state {
        if !states.contains(id) {
            return Err(SimulationError::UnknownInitialState(id.clone()));
        }
        ensure_finite(id, *value)?;
    }
    for (id, value) in &request.parameter_overrides {
        if !world.parameters().contains_key(id) {
            return Err(SimulationError::UnknownParameterOverride(id.clone()));
        }
        ensure_finite(id, *value)?;
    }
    for (id, value) in &request.inputs {
        match world.variables().get(id) {
            Some(variable) if variable.role != VariableRole::State => {}
            Some(_) => return Err(SimulationError::InputTargetsState(id.clone())),
            None => return Err(SimulationError::UnknownInput(id.clone())),
        }
        ensure_finite(id, *value)?;
    }
    for change in &request.scheduled_parameters {
        if !world.parameters().contains_key(&change.id) {
            return Err(SimulationError::UnknownParameterOverride(change.id.clone()));
        }
        ensure_intervention(change)?;
    }
    for change in &request.scheduled_inputs {
        match world.variables().get(&change.id) {
            Some(variable) if variable.role != VariableRole::State => {}
            Some(_) => return Err(SimulationError::InputTargetsState(change.id.clone())),
            None => return Err(SimulationError::UnknownInput(change.id.clone())),
        }
        ensure_intervention(change)?;
    }
    let parameters = world
        .parameters()
        .iter()
        .map(|(id, parameter)| {
            (
                id.clone(),
                request
                    .parameter_overrides
                    .get(id)
                    .copied()
                    .unwrap_or(parameter.value),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let compiled = CompiledDiscreteWorld::compile(world);
    let mut state = request.initial_state.clone();
    let mut trajectory = Trajectory::from_initial(config.start, &state);
    for step in 1..=config.steps {
        let time = config.start + (step - 1) as f64;
        let parameters = request.parameter_values_at(&parameters, time);
        let inputs = request.input_values_at(time);
        let next = evaluate_discrete(
            &compiled,
            &SimulationContext::new(state.clone(), parameters, inputs),
        )?;
        state = next;
        trajectory.push(config.start + step as f64, &state);
    }
    Ok(trajectory)
}

fn ensure_finite(name: &Identifier, value: f64) -> Result<(), SimulationError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(SimulationError::NonFiniteInput {
            name: name.clone(),
            value,
        })
    }
}

fn ensure_intervention(change: &crate::ScheduledValue) -> Result<(), SimulationError> {
    if !change.time.is_finite() {
        return Err(SimulationError::InvalidInterventionTime {
            name: change.id.clone(),
            time: change.time,
        });
    }
    ensure_finite(&change.id, change.value)
}

#[cfg(test)]
mod tests {
    use lawsynth_expr::Expr;
    use lawsynth_world::{DiscreteLaw, DiscreteWorld, Variable, VariableRole};

    use super::*;

    #[test]
    fn simulates_a_discrete_recurrence() {
        let id = |value| Identifier::new(value).unwrap();
        let world = DiscreteWorld::new(
            [Variable::new(id("x"), VariableRole::State)],
            [],
            [DiscreteLaw::new(
                id("x"),
                Expr::sum(Expr::symbol(id("x")), Expr::constant(1.0)),
            )],
        )
        .unwrap();
        let trajectory = simulate_discrete(
            &world,
            DiscreteSimulationConfig::new(0.0, 3).unwrap(),
            &SimulationRequest::default().with_initial(id("x"), 2.0),
        )
        .unwrap();
        assert_eq!(trajectory.values[&id("x")], vec![2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn applies_scheduled_discrete_parameter_changes() {
        let id = |value| Identifier::new(value).unwrap();
        let world = DiscreteWorld::new(
            [Variable::new(id("x"), VariableRole::State)],
            [lawsynth_world::Parameter::new(id("increment"), 1.0)],
            [DiscreteLaw::new(
                id("x"),
                Expr::sum(Expr::symbol(id("x")), Expr::symbol(id("increment"))),
            )],
        )
        .unwrap();
        let request = SimulationRequest::default()
            .with_initial(id("x"), 0.0)
            .with_scheduled_parameter(1.0, id("increment"), 2.0);
        let trajectory = simulate_discrete(
            &world,
            DiscreteSimulationConfig::new(0.0, 3).unwrap(),
            &request,
        )
        .unwrap();
        assert_eq!(trajectory.values[&id("x")], vec![0.0, 1.0, 3.0, 5.0]);
    }
}
