use std::collections::{BTreeMap, BTreeSet};

use lawsynth_core::Identifier;
use lawsynth_world::{VariableRole, World};

use crate::{
    CompiledContinuousWorld, SimulationConfig, SimulationContext, SimulationError,
    SimulationRequest, Trajectory, evaluate_continuous, state::offset_state,
};

/// Simulates a continuous World using classical fourth-order Runge-Kutta.
pub fn simulate(
    world: &World,
    config: SimulationConfig,
    request: &SimulationRequest,
) -> Result<Trajectory, SimulationError> {
    let state_ids: BTreeSet<_> = world.state_ids().cloned().collect();
    for id in &state_ids {
        let value = request
            .initial_state
            .get(id)
            .ok_or_else(|| SimulationError::MissingInitialState(id.clone()))?;
        ensure_finite(id, *value)?;
    }
    for (id, value) in &request.initial_state {
        if !state_ids.contains(id) {
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
            (id.clone(), request.parameter_overrides.get(id).copied().unwrap_or(parameter.value))
        })
        .collect();
    let compiled = CompiledContinuousWorld::compile(world);
    let mut state = request.initial_state.clone();
    let mut trajectory = Trajectory::from_initial(config.start, &state);
    let mut time = config.start;

    // Repeated `time = time + step` additions drift from the mathematically exact
    // grid by a few ULPs per step, so after enough steps the loop's final
    // iteration can see a `remaining` that is tiny-but-nonzero floating-point
    // noise rather than a real leftover interval. A tolerance scaled to the raw
    // f64 epsilon (`1e-12`) is tighter than that accumulated drift and lets an
    // extra, near-zero-duration step sneak in right at the end, producing a
    // spurious duplicate-looking final sample. Scale the tolerance to the step
    // size instead so it comfortably absorbs summation drift while staying far
    // smaller than any real remaining sub-step.
    const END_TOLERANCE_FACTOR: f64 = 1e-6;
    while time < config.end {
        let remaining = config.end - time;
        if remaining <= config.step * END_TOLERANCE_FACTOR {
            break;
        }
        let scheduled_remaining = request
            .next_change_after(time, config.end)
            .map(|event_time| event_time - time)
            .unwrap_or(remaining);
        let step = remaining.min(config.step).min(scheduled_remaining);
        state = rk4_step(&compiled, &state, &parameters, request, time, step)?;
        let next_time = time + step;
        if next_time <= time {
            return Err(SimulationError::TimeResolutionLoss);
        }
        time = if (config.end - next_time).abs() <= config.step * END_TOLERANCE_FACTOR {
            config.end
        } else {
            next_time
        };
        trajectory.push(time, &state);
    }
    Ok(trajectory)
}

fn rk4_step(
    compiled: &CompiledContinuousWorld,
    state: &BTreeMap<Identifier, f64>,
    parameters: &BTreeMap<Identifier, f64>,
    request: &SimulationRequest,
    time: f64,
    step: f64,
) -> Result<BTreeMap<Identifier, f64>, SimulationError> {
    let k1 = derivatives(compiled, state, parameters, request, time, true)?;
    let k2 = derivatives(
        compiled,
        &offset_state(state, &k1, step / 2.0),
        parameters,
        request,
        time + step / 2.0,
        true,
    )?;
    let k3 = derivatives(
        compiled,
        &offset_state(state, &k2, step / 2.0),
        parameters,
        request,
        time + step / 2.0,
        true,
    )?;
    let k4 = derivatives(
        compiled,
        &offset_state(state, &k3, step),
        parameters,
        request,
        time + step,
        false,
    )?;

    state
        .iter()
        .map(|(id, value)| {
            let next = value + step * (k1[id] + 2.0 * k2[id] + 2.0 * k3[id] + k4[id]) / 6.0;
            ensure_finite(id, next)?;
            Ok((id.clone(), next))
        })
        .collect()
}

fn derivatives(
    compiled: &CompiledContinuousWorld,
    state: &BTreeMap<Identifier, f64>,
    parameters: &BTreeMap<Identifier, f64>,
    request: &SimulationRequest,
    time: f64,
    include_changes_at_time: bool,
) -> Result<BTreeMap<Identifier, f64>, SimulationError> {
    let parameters = if include_changes_at_time {
        request.parameter_values_at(parameters, time)
    } else {
        request.parameter_values_before(parameters, time)
    };
    let inputs = if include_changes_at_time {
        request.input_values_at(time)
    } else {
        request.input_values_before(time)
    };
    evaluate_continuous(compiled, &SimulationContext::new(state.clone(), parameters, inputs))
}

fn ensure_finite(name: &Identifier, value: f64) -> Result<(), SimulationError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(SimulationError::NonFiniteInput { name: name.clone(), value })
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
    use lawsynth_world::{ContinuousLaw, Parameter, Variable, VariableRole};

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn simulates_exponential_decay_with_rk4_accuracy() {
        let world = World::new(
            [Variable::new(id("x"), VariableRole::State)],
            [Parameter::new(id("rate"), 1.0)],
            [ContinuousLaw::new(
                id("x"),
                Expr::product(
                    Expr::constant(-1.0),
                    Expr::product(Expr::symbol(id("rate")), Expr::symbol(id("x"))),
                ),
            )],
        )
        .unwrap();
        let trajectory = simulate(
            &world,
            SimulationConfig::new(0.0, 1.0, 0.01).unwrap(),
            &SimulationRequest::default().with_initial(id("x"), 1.0),
        )
        .unwrap();
        let final_value = trajectory.values[&id("x")].last().copied().unwrap();
        assert!((final_value - (-1.0_f64).exp()).abs() < 1e-8);
    }

    #[test]
    fn sample_count_matches_the_requested_grid_despite_float_drift() {
        let world = World::new(
            [Variable::new(id("x"), VariableRole::State)],
            [Parameter::new(id("rate"), 1.0)],
            [ContinuousLaw::new(
                id("x"),
                Expr::product(
                    Expr::constant(-1.0),
                    Expr::product(Expr::symbol(id("rate")), Expr::symbol(id("x"))),
                ),
            )],
        )
        .unwrap();
        let trajectory = simulate(
            &world,
            SimulationConfig::new(0.0, 5.0, 0.01).unwrap(),
            &SimulationRequest::default().with_initial(id("x"), 1.0),
        )
        .unwrap();
        // 0.0..=5.0 in steps of 0.01 is exactly 500 steps -> 501 samples including t=0.
        // Floating-point summation drift in `time` must not cause an extra, near-zero
        // final step to sneak in.
        assert_eq!(trajectory.samples(), 501);
        assert_eq!(*trajectory.time.last().unwrap(), 5.0);
    }

    #[test]
    fn applies_a_parameter_intervention() {
        let world = World::new(
            [Variable::new(id("x"), VariableRole::State)],
            [Parameter::new(id("rate"), 1.0)],
            [ContinuousLaw::new(
                id("x"),
                Expr::product(Expr::symbol(id("rate")), Expr::symbol(id("x"))),
            )],
        )
        .unwrap();
        let request = SimulationRequest::default()
            .with_initial(id("x"), 1.0)
            .with_parameter_override(id("rate"), 2.0);
        let trajectory =
            simulate(&world, SimulationConfig::new(0.0, 1.0, 0.01).unwrap(), &request).unwrap();
        assert!((trajectory.values[&id("x")].last().unwrap() - 2.0_f64.exp()).abs() < 2e-7);
    }

    #[test]
    fn applies_scheduled_parameter_changes_at_an_exact_step_boundary() {
        let world = World::new(
            [Variable::new(id("x"), VariableRole::State)],
            [Parameter::new(id("rate"), 1.0)],
            [ContinuousLaw::new(
                id("x"),
                Expr::product(Expr::symbol(id("rate")), Expr::constant(1.0)),
            )],
        )
        .unwrap();
        let request = SimulationRequest::default()
            .with_initial(id("x"), 0.0)
            .with_scheduled_parameter(0.5, id("rate"), 3.0);
        let trajectory =
            simulate(&world, SimulationConfig::new(0.0, 1.0, 1.0).unwrap(), &request).unwrap();
        assert_eq!(trajectory.time, vec![0.0, 0.5, 1.0]);
        assert!((trajectory.values[&id("x")][2] - 2.0).abs() < 1e-12);
    }

    #[test]
    fn accepts_typed_world_interventions() {
        let world = World::new(
            [Variable::new(id("x"), VariableRole::State)],
            [Parameter::new(id("rate"), 1.0)],
            [ContinuousLaw::new(id("x"), Expr::symbol(id("rate")))],
        )
        .unwrap();
        let request = SimulationRequest::default()
            .with_initial(id("x"), 0.0)
            .with_intervention(lawsynth_world::Intervention::parameter(0.5, id("rate"), 3.0));
        let trajectory =
            simulate(&world, SimulationConfig::new(0.0, 1.0, 1.0).unwrap(), &request).unwrap();
        assert!((trajectory.values[&id("x")][2] - 2.0).abs() < 1e-12);
    }
}
