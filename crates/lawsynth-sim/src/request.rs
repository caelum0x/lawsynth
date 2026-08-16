use std::collections::BTreeMap;

use lawsynth_core::Identifier;
use lawsynth_world::{Intervention, InterventionTarget};

use crate::SimulationError;

/// An inclusive time interval sampled at a fixed maximum step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimulationConfig {
    pub start: f64,
    pub end: f64,
    pub step: f64,
}

/// Numbered time grid for a discrete recurrence simulation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiscreteSimulationConfig {
    pub start: f64,
    pub steps: usize,
}

impl DiscreteSimulationConfig {
    pub fn new(start: f64, steps: usize) -> Result<Self, SimulationError> {
        if !start.is_finite() {
            return Err(SimulationError::InvalidTimeGrid);
        }
        Ok(Self { start, steps })
    }
}

impl SimulationConfig {
    pub fn new(start: f64, end: f64, step: f64) -> Result<Self, SimulationError> {
        let config = Self { start, end, step };
        if !start.is_finite()
            || !end.is_finite()
            || !step.is_finite()
            || end <= start
            || step <= 0.0
        {
            return Err(SimulationError::InvalidTimeGrid);
        }
        Ok(config)
    }
}

/// A value change which becomes active at an inclusive simulation timestamp.
#[derive(Clone, Debug, PartialEq)]
pub struct ScheduledValue {
    pub time: f64,
    pub id: Identifier,
    pub value: f64,
}

/// Inputs and interventions that define one scenario for a world simulation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SimulationRequest {
    pub initial_state: BTreeMap<Identifier, f64>,
    pub parameter_overrides: BTreeMap<Identifier, f64>,
    pub inputs: BTreeMap<Identifier, f64>,
    pub scheduled_parameters: Vec<ScheduledValue>,
    pub scheduled_inputs: Vec<ScheduledValue>,
}

impl SimulationRequest {
    pub fn with_initial(mut self, id: Identifier, value: f64) -> Self {
        self.initial_state.insert(id, value);
        self
    }

    pub fn with_parameter_override(mut self, id: Identifier, value: f64) -> Self {
        self.parameter_overrides.insert(id, value);
        self
    }

    pub fn with_input(mut self, id: Identifier, value: f64) -> Self {
        self.inputs.insert(id, value);
        self
    }

    /// Schedules a parameter value to take effect at `time`.
    pub fn with_scheduled_parameter(mut self, time: f64, id: Identifier, value: f64) -> Self {
        self.scheduled_parameters
            .push(ScheduledValue { time, id, value });
        self
    }

    /// Schedules a control/input value to take effect at `time`.
    pub fn with_scheduled_input(mut self, time: f64, id: Identifier, value: f64) -> Self {
        self.scheduled_inputs
            .push(ScheduledValue { time, id, value });
        self
    }

    /// Adds a typed World-IR intervention to this simulation scenario.
    pub fn with_intervention(mut self, intervention: Intervention) -> Self {
        match intervention.target {
            InterventionTarget::Parameter(id) => {
                self.scheduled_parameters.push(ScheduledValue {
                    time: intervention.time,
                    id,
                    value: intervention.value,
                });
            }
            InterventionTarget::Input(id) => {
                self.scheduled_inputs.push(ScheduledValue {
                    time: intervention.time,
                    id,
                    value: intervention.value,
                });
            }
        }
        self
    }

    pub(crate) fn parameter_values_at(
        &self,
        base: &BTreeMap<Identifier, f64>,
        time: f64,
    ) -> BTreeMap<Identifier, f64> {
        values_at(base, &self.scheduled_parameters, time)
    }

    pub(crate) fn input_values_at(&self, time: f64) -> BTreeMap<Identifier, f64> {
        values_at(&self.inputs, &self.scheduled_inputs, time)
    }

    pub(crate) fn parameter_values_before(
        &self,
        base: &BTreeMap<Identifier, f64>,
        time: f64,
    ) -> BTreeMap<Identifier, f64> {
        values_before(base, &self.scheduled_parameters, time)
    }

    pub(crate) fn input_values_before(&self, time: f64) -> BTreeMap<Identifier, f64> {
        values_before(&self.inputs, &self.scheduled_inputs, time)
    }

    pub(crate) fn next_change_after(&self, time: f64, end: f64) -> Option<f64> {
        self.scheduled_parameters
            .iter()
            .chain(&self.scheduled_inputs)
            .filter_map(|change| {
                (change.time > time && change.time < end && change.time.is_finite())
                    .then_some(change.time)
            })
            .min_by(f64::total_cmp)
    }
}

fn values_at(
    base: &BTreeMap<Identifier, f64>,
    scheduled: &[ScheduledValue],
    time: f64,
) -> BTreeMap<Identifier, f64> {
    values_with(base, scheduled, |change| change.time <= time)
}

fn values_before(
    base: &BTreeMap<Identifier, f64>,
    scheduled: &[ScheduledValue],
    time: f64,
) -> BTreeMap<Identifier, f64> {
    values_with(base, scheduled, |change| change.time < time)
}

fn values_with(
    base: &BTreeMap<Identifier, f64>,
    scheduled: &[ScheduledValue],
    predicate: impl Fn(&ScheduledValue) -> bool,
) -> BTreeMap<Identifier, f64> {
    let mut values = base.clone();
    let mut scheduled = scheduled
        .iter()
        .filter(|change| predicate(change))
        .collect::<Vec<_>>();
    scheduled.sort_by(|left, right| {
        left.time
            .total_cmp(&right.time)
            .then_with(|| left.id.cmp(&right.id))
    });
    for change in scheduled {
        values.insert(change.id.clone(), change.value);
    }
    values
}
