//! Deterministic forward simulation and validation of discovered controlled
//! (SINDYc) models.
//!
//! Discovery ([`discover_controlled`](crate::discover_controlled)) fits a model
//! `ẋ = Θ(x, u) Ξ` but never rolls it forward. This module closes the loop:
//! [`simulate_controlled`] integrates a [`ControlledModel`] under a supplied
//! control signal with fixed-step RK4, and [`validate_controlled`] scores that
//! rollout against held-out data with predictive R² and RMSE.
//!
//! # How the right-hand side is evaluated
//!
//! The model carries the structured augmented library
//! ([`ControlledModel::library`]). At every RK4 stage we bind the current
//! `(state, control)` values into an [`Environment`] and evaluate each library
//! term's **expression tree** with [`lawsynth_expr::evaluate`] — never by
//! re-parsing the human-readable label strings. Term `k`'s value is then
//! multiplied by `equation.coefficients[k]` and summed, giving
//! `ẋ_i = Σ_k coef[i][k] · term_k(x, u)` exactly as fitted.
//!
//! # Control interpolation rule (deterministic)
//!
//! RK4 evaluates the right-hand side at the stage times `t`, `t + dt/2`, and
//! `t + dt`. A control supplied as a closure is evaluated exactly at those
//! times. A control supplied as sampled columns is **linearly interpolated**
//! between adjacent samples, and **clamped (held constant)** outside the sampled
//! range. Linear interpolation is deterministic and continuous, so identical
//! inputs yield bit-identical stage values and hence a bit-identical trajectory.
//!
//! # Honest limits
//!
//! - This is an **open-loop** rollout: model-coefficient error accumulates over
//!   long horizons, so predictive R² degrades as the horizon grows.
//! - The integrator is **fixed-step RK4 only** — there is no adaptive stepping
//!   and no stiffness handling. Stiff systems need a small `dt`.
//! - Sampled controls must cover the simulated horizon on a compatible grid;
//!   values requested outside the samples are clamped, not extrapolated.

use std::collections::BTreeMap;

use lawsynth_core::Identifier;
use lawsynth_data::Dataset;
use lawsynth_expr::{Environment, evaluate};
use lawsynth_score::fit_statistics;

use crate::{ControlError, ControlSpec, ControlledModel};

/// An exogenous control signal supplied to [`simulate_controlled`].
///
/// A signal names the control channels it drives (which must match the model's
/// controls, in order) and produces a value per channel at any query time,
/// either from sampled columns (linearly interpolated) or from a deterministic
/// closure. See the [module docs](crate::simulate) for the interpolation rule.
pub struct ControlSignal {
    controls: Vec<Identifier>,
    source: ControlSource,
}

enum ControlSource {
    /// One sampled column per control on a shared, ascending time axis.
    Sampled { time: Vec<f64>, columns: Vec<Vec<f64>> },
    /// A deterministic function of time returning one value per control.
    Function(Box<dyn Fn(f64) -> Vec<f64>>),
}

impl std::fmt::Debug for ControlSignal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The boxed closure is not printable; report its shape instead so the
        // type still satisfies `Debug` (needed by `Result::unwrap_err` in tests).
        let kind = match &self.source {
            ControlSource::Sampled { time, .. } => {
                format!("sampled({} points)", time.len())
            }
            ControlSource::Function(_) => "function".to_string(),
        };
        formatter
            .debug_struct("ControlSignal")
            .field("controls", &self.controls)
            .field("source", &kind)
            .finish()
    }
}

impl ControlSignal {
    /// Builds a sampled control signal from a shared time axis and one column
    /// per control, in the control order used by the model.
    ///
    /// Fails when there are no controls, when the column count differs from the
    /// control count, when any column length differs from the time-axis length,
    /// when the time axis is empty, or when it is not strictly ascending (a
    /// prerequisite for deterministic interpolation).
    pub fn sampled(
        controls: impl IntoIterator<Item = Identifier>,
        time: Vec<f64>,
        columns: Vec<Vec<f64>>,
    ) -> Result<Self, ControlError> {
        let controls = controls.into_iter().collect::<Vec<_>>();
        if controls.is_empty() {
            return Err(ControlError::ControlGrid(
                "a control signal needs at least one control".into(),
            ));
        }
        if columns.len() != controls.len() {
            return Err(ControlError::ControlGrid(format!(
                "{} controls but {} sampled columns",
                controls.len(),
                columns.len()
            )));
        }
        if time.is_empty() {
            return Err(ControlError::ControlGrid("control time axis is empty".into()));
        }
        for (index, column) in columns.iter().enumerate() {
            if column.len() != time.len() {
                return Err(ControlError::ControlGrid(format!(
                    "control column {index} has {} samples but the time axis has {}",
                    column.len(),
                    time.len()
                )));
            }
        }
        if !time.windows(2).all(|pair| pair[1] > pair[0]) {
            return Err(ControlError::ControlGrid(
                "control time axis must be strictly ascending".into(),
            ));
        }
        Ok(Self { controls, source: ControlSource::Sampled { time, columns } })
    }

    /// Builds a control signal from a deterministic closure `t ↦ [u₁(t), …]`.
    ///
    /// The closure must return exactly one value per control, in control order,
    /// on every call; this is checked at each stage during simulation.
    pub fn from_fn<F>(
        controls: impl IntoIterator<Item = Identifier>,
        function: F,
    ) -> Result<Self, ControlError>
    where
        F: Fn(f64) -> Vec<f64> + 'static,
    {
        let controls = controls.into_iter().collect::<Vec<_>>();
        if controls.is_empty() {
            return Err(ControlError::ControlGrid(
                "a control signal needs at least one control".into(),
            ));
        }
        Ok(Self { controls, source: ControlSource::Function(Box::new(function)) })
    }

    /// The control channels this signal drives, in order.
    pub fn controls(&self) -> &[Identifier] {
        &self.controls
    }

    /// Evaluates every control channel at `t`, returning values in control order.
    fn sample(&self, t: f64) -> Result<Vec<f64>, ControlError> {
        match &self.source {
            ControlSource::Sampled { time, columns } => {
                Ok(columns.iter().map(|column| interpolate(time, column, t)).collect())
            }
            ControlSource::Function(function) => {
                let values = function(t);
                if values.len() != self.controls.len() {
                    return Err(ControlError::ControlGrid(format!(
                        "control closure returned {} values but the model has {} controls",
                        values.len(),
                        self.controls.len()
                    )));
                }
                if let Some(bad) = values.iter().find(|value| !value.is_finite()) {
                    return Err(ControlError::Simulation(format!(
                        "control closure returned a non-finite value {bad} at t = {t}"
                    )));
                }
                Ok(values)
            }
        }
    }
}

/// Linearly interpolates `column` (sampled at ascending `time`) at query `t`.
///
/// Outside the sampled range the nearest endpoint value is held constant
/// (clamped), matching the documented interpolation rule. `time` is guaranteed
/// non-empty and strictly ascending by [`ControlSignal::sampled`].
fn interpolate(time: &[f64], column: &[f64], t: f64) -> f64 {
    if t <= time[0] {
        return column[0];
    }
    let last = time.len() - 1;
    if t >= time[last] {
        return column[last];
    }
    // Ascending axis: find the bracketing interval [time[i], time[i + 1]].
    let upper = time.partition_point(|sample| *sample <= t);
    let lower = upper - 1;
    let span = time[upper] - time[lower];
    let weight = (t - time[lower]) / span;
    column[lower] + weight * (column[upper] - column[lower])
}

/// Fixed-step RK4 configuration for [`simulate_controlled`].
///
/// The rollout starts at `t0`, takes `steps` steps of size `dt`, and returns
/// `steps + 1` samples (the initial state plus one per step).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimConfig {
    /// Simulation start time (also the time of the supplied initial state).
    pub t0: f64,
    /// Fixed step size. Must be finite and strictly positive.
    pub dt: f64,
    /// Number of RK4 steps to take. Must be at least one.
    pub steps: usize,
}

impl SimConfig {
    /// Builds a validated fixed-step configuration.
    pub fn new(t0: f64, dt: f64, steps: usize) -> Result<Self, ControlError> {
        if !dt.is_finite() || dt <= 0.0 {
            return Err(ControlError::ControlGrid(format!(
                "step size must be positive and finite, got {dt}"
            )));
        }
        if !t0.is_finite() {
            return Err(ControlError::ControlGrid(format!("start time must be finite, got {t0}")));
        }
        if steps == 0 {
            return Err(ControlError::ControlGrid("simulation needs at least one step".into()));
        }
        Ok(Self { t0, dt, steps })
    }
}

/// A simulated state trajectory in deterministic state-id order.
///
/// `time` has `steps + 1` entries; each state column in `values` has the same
/// length and is keyed by its state identifier.
#[derive(Clone, Debug, PartialEq)]
pub struct Trajectory {
    /// Sample times, ascending, length `steps + 1`.
    pub time: Vec<f64>,
    /// One column per state, keyed by identifier, each aligned to `time`.
    pub values: BTreeMap<Identifier, Vec<f64>>,
}

impl Trajectory {
    /// Returns the sampled column for `state`, if the trajectory carries it.
    pub fn column(&self, state: &Identifier) -> Option<&[f64]> {
        self.values.get(state).map(Vec::as_slice)
    }

    /// Number of samples (`steps + 1`).
    pub fn samples(&self) -> usize {
        self.time.len()
    }
}

/// Predictive fit of one state's simulated column against its observations.
#[derive(Clone, Debug, PartialEq)]
pub struct StateScore {
    /// The state this score describes.
    pub state: Identifier,
    /// Coefficient of determination `1 − SS_res/SS_tot` for this state.
    pub r_squared: f64,
    /// Root-mean-squared error between simulated and observed values.
    pub rmse: f64,
}

/// The result of [`validate_controlled`]: per-state and aggregate predictive fit.
///
/// The aggregate is computed by pooling every state's samples (in the model's
/// state order) into one observed/predicted pair and scoring that, so it weights
/// each sample equally across states.
#[derive(Clone, Debug, PartialEq)]
pub struct ControlScore {
    /// Per-state predictive fit, in the model's state order.
    pub per_state: Vec<StateScore>,
    /// Pooled coefficient of determination across all states.
    pub aggregate_r_squared: f64,
    /// Pooled root-mean-squared error across all states.
    pub aggregate_rmse: f64,
}

impl ControlScore {
    /// Looks up the predictive fit for a given state.
    pub fn state_score(&self, state: &Identifier) -> Option<&StateScore> {
        self.per_state.iter().find(|score| &score.state == state)
    }
}

/// Integrates a discovered controlled model forward under `control`.
///
/// Rolls `ẋ = Θ(x, u) Ξ` from `initial` (one value per state, in the model's
/// state order) using fixed-step RK4, evaluating the control at each RK4 stage
/// per the documented interpolation rule. Returns the state trajectory as
/// columns.
///
/// # Determinism
///
/// Every operation is a pure `f64` computation over deterministically ordered
/// data (library terms fixed at discovery, states/controls in model order,
/// `BTreeMap` environments). Identical `(model, initial, control, config)`
/// inputs therefore yield a **bit-identical** [`Trajectory`].
///
/// # Errors
///
/// - [`ControlError::InitialStateDimension`] if `initial.len()` ≠ state count.
/// - [`ControlError::ControlMismatch`] if `control`'s channels are not the
///   model's controls, in order.
/// - [`ControlError::ControlGrid`] for an invalid `config`.
/// - [`ControlError::Simulation`] if a stage evaluates to a non-finite value.
pub fn simulate_controlled(
    model: &ControlledModel,
    initial: &[f64],
    control: &ControlSignal,
    config: &SimConfig,
) -> Result<Trajectory, ControlError> {
    if initial.len() != model.states.len() {
        return Err(ControlError::InitialStateDimension {
            expected: model.states.len(),
            found: initial.len(),
        });
    }
    if control.controls() != model.controls.as_slice() {
        return Err(ControlError::ControlMismatch {
            expected: model.controls.iter().map(|id| id.to_string()).collect(),
            found: control.controls().iter().map(|id| id.to_string()).collect(),
        });
    }
    for (state, value) in model.states.iter().zip(initial) {
        if !value.is_finite() {
            return Err(ControlError::Simulation(format!(
                "initial value for state '{state}' is non-finite: {value}"
            )));
        }
    }

    // Only terms that some equation actually uses need evaluating; skipping the
    // rest is deterministic and avoids spurious evaluation of unused terms.
    let active = active_term_indices(model);

    let mut state = initial.to_vec();
    let mut time = Vec::with_capacity(config.steps + 1);
    let mut columns: Vec<Vec<f64>> =
        model.states.iter().map(|_| Vec::with_capacity(config.steps + 1)).collect();

    let mut t = config.t0;
    record(&mut time, &mut columns, t, &state);
    for _ in 0..config.steps {
        state = rk4_step(model, &active, &state, control, t, config.dt)?;
        t += config.dt;
        for (value, state_id) in state.iter().zip(&model.states) {
            if !value.is_finite() {
                return Err(ControlError::Simulation(format!(
                    "state '{state_id}' became non-finite ({value}) at t = {t}"
                )));
            }
        }
        record(&mut time, &mut columns, t, &state);
    }

    let values =
        model.states.iter().cloned().zip(columns).collect::<BTreeMap<Identifier, Vec<f64>>>();
    Ok(Trajectory { time, values })
}

/// Options for [`validate_controlled`].
///
/// The one knob is `substeps`: how many fixed-step RK4 steps to take *between*
/// consecutive observed samples before comparing at the sample points. A larger
/// value shrinks the integrator's own step error, so the resulting score
/// reflects the *model's* error rather than the integrator's. The default of one
/// integrates exactly on the data grid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ValidationConfig {
    /// RK4 sub-steps taken between consecutive observed samples. Must be ≥ 1.
    pub substeps: usize,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self { substeps: 1 }
    }
}

/// Simulates the model from a dataset's own initial condition and control, then
/// scores the rollout against the dataset's observed states.
///
/// The control is taken from the dataset's control columns (sampled on the
/// dataset's time grid and linearly interpolated at RK4 stage times), and the
/// initial state is the first row of each state column. Integration uses
/// fixed-step RK4 with `config.substeps` steps between consecutive observed
/// samples; the rollout is compared to the observations at the sample points.
/// Reports per-state and aggregate predictive R² and RMSE via
/// [`lawsynth_score::fit_statistics`].
///
/// This assumes an (approximately) regular time grid: a single uniform step is
/// used so that every `substeps`-th simulated sample lands on an observed time.
///
/// # Errors
///
/// - [`ControlError::ControlMismatch`] if `spec` disagrees with the model.
/// - [`ControlError::UnknownIdentifier`] if a designated column is missing.
/// - [`ControlError::ControlGrid`] if the dataset has fewer than two samples or
///   `config.substeps` is zero.
/// - Any error from [`simulate_controlled`] or from scoring.
pub fn validate_controlled(
    model: &ControlledModel,
    dataset: &Dataset,
    spec: &ControlSpec,
    config: &ValidationConfig,
) -> Result<ControlScore, ControlError> {
    if spec.states() != model.states.as_slice() || spec.controls() != model.controls.as_slice() {
        return Err(ControlError::ControlMismatch {
            expected: model.states.iter().chain(&model.controls).map(|id| id.to_string()).collect(),
            found: spec.states().iter().chain(spec.controls()).map(|id| id.to_string()).collect(),
        });
    }
    spec.validate_against(dataset)?;

    if config.substeps == 0 {
        return Err(ControlError::ControlGrid("validation substeps must be at least one".into()));
    }

    let time = dataset.time().values();
    if time.len() < 2 {
        return Err(ControlError::ControlGrid(
            "validation needs at least two samples to form a step".into(),
        ));
    }

    let columns = dataset.columns();
    let initial = model.states.iter().map(|state| columns[state].values[0]).collect::<Vec<f64>>();

    let control_columns = model
        .controls
        .iter()
        .map(|control| columns[control].values.clone())
        .collect::<Vec<Vec<f64>>>();
    let control = ControlSignal::sampled(model.controls.clone(), time.to_vec(), control_columns)?;

    // One uniform step spanning the whole horizon, subdivided `substeps` times
    // per observed interval, so index `i * substeps` lands on observed time `i`.
    let intervals = time.len() - 1;
    let total_steps = intervals * config.substeps;
    let step = (time[time.len() - 1] - time[0]) / total_steps as f64;
    let sim_config = SimConfig::new(time[0], step, total_steps)?;
    let trajectory = simulate_controlled(model, &initial, &control, &sim_config)?;

    let mut per_state = Vec::with_capacity(model.states.len());
    let mut pooled_observed = Vec::new();
    let mut pooled_predicted = Vec::new();
    for state in &model.states {
        let observed = columns[state].values.as_slice();
        let dense = trajectory.column(state).expect("trajectory carries every state");
        let predicted = (0..time.len()).map(|i| dense[i * config.substeps]).collect::<Vec<f64>>();
        let stats = fit_statistics(observed, &predicted)?;
        per_state.push(StateScore {
            state: state.clone(),
            r_squared: stats.r_squared,
            rmse: stats.root_mean_squared_error,
        });
        pooled_observed.extend_from_slice(observed);
        pooled_predicted.extend_from_slice(&predicted);
    }
    let aggregate = fit_statistics(&pooled_observed, &pooled_predicted)?;

    Ok(ControlScore {
        per_state,
        aggregate_r_squared: aggregate.r_squared,
        aggregate_rmse: aggregate.root_mean_squared_error,
    })
}

/// Indices of library terms referenced by at least one equation (non-zero coef).
fn active_term_indices(model: &ControlledModel) -> Vec<usize> {
    (0..model.library_terms.len())
        .filter(|&k| model.equations.iter().any(|equation| equation.coefficients[k] != 0.0))
        .collect()
}

/// One classical fourth-order Runge-Kutta step from `state` at time `t`.
fn rk4_step(
    model: &ControlledModel,
    active: &[usize],
    state: &[f64],
    control: &ControlSignal,
    t: f64,
    dt: f64,
) -> Result<Vec<f64>, ControlError> {
    let k1 = rhs(model, active, state, control, t)?;
    let s2 = axpy(state, dt * 0.5, &k1);
    let k2 = rhs(model, active, &s2, control, t + 0.5 * dt)?;
    let s3 = axpy(state, dt * 0.5, &k2);
    let k3 = rhs(model, active, &s3, control, t + 0.5 * dt)?;
    let s4 = axpy(state, dt, &k3);
    let k4 = rhs(model, active, &s4, control, t + dt)?;

    let next = state
        .iter()
        .enumerate()
        .map(|(i, value)| value + dt / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]))
        .collect();
    Ok(next)
}

/// Computes `base + scale · direction` component-wise.
fn axpy(base: &[f64], scale: f64, direction: &[f64]) -> Vec<f64> {
    base.iter().zip(direction).map(|(b, d)| b + scale * d).collect()
}

/// Evaluates the model's right-hand side `ẋ_i = Σ_k coef[i][k] · term_k(x, u)`.
///
/// Library terms are evaluated **structurally** from their stored expression
/// trees against a `(state, control)` environment — no string parsing.
fn rhs(
    model: &ControlledModel,
    active: &[usize],
    state: &[f64],
    control: &ControlSignal,
    t: f64,
) -> Result<Vec<f64>, ControlError> {
    let controls = control.sample(t)?;

    let mut environment: Environment = Environment::new();
    for (id, value) in model.states.iter().zip(state) {
        environment.insert(id.clone(), *value);
    }
    for (id, value) in model.controls.iter().zip(&controls) {
        environment.insert(id.clone(), *value);
    }

    let terms = model.library.terms();
    let mut term_values = vec![0.0_f64; terms.len()];
    for &k in active {
        term_values[k] = evaluate(&terms[k].expression, &environment)
            .map_err(|error| ControlError::Simulation(error.to_string()))?;
    }

    let derivatives = model
        .equations
        .iter()
        .map(|equation| {
            equation
                .coefficients
                .iter()
                .enumerate()
                .filter(|(_, coefficient)| **coefficient != 0.0)
                .map(|(k, coefficient)| coefficient * term_values[k])
                .sum::<f64>()
        })
        .collect();
    Ok(derivatives)
}

fn record(time: &mut Vec<f64>, columns: &mut [Vec<f64>], t: f64, state: &[f64]) {
    time.push(t);
    for (column, value) in columns.iter_mut().zip(state) {
        column.push(*value);
    }
}
