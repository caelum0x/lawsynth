//! The trajectory produced by a forward-sensitivity integration.

use std::fmt::Write as _;

use lawsynth_core::Identifier;

/// The integrated state trajectory together with the trajectory sensitivities
/// `S_j(t) = ∂x(t)/∂θ_j` for every discovered parameter.
///
/// All three time-indexed collections share the same time grid: `times[k]` is
/// the instant of the `k`-th sample, `state[k]` is `x(times[k])` in `states`
/// order, and `sensitivities[j][k]` is `S_j(times[k]) = ∂x(times[k])/∂θ_j`, also
/// in `states` order. There are `steps + 1` samples, the first at `t0`.
#[derive(Clone, Debug, PartialEq)]
pub struct SensitivityTrajectory {
    states: Vec<Identifier>,
    parameters: Vec<Identifier>,
    times: Vec<f64>,
    state: Vec<Vec<f64>>,
    sensitivities: Vec<Vec<Vec<f64>>>,
}

impl SensitivityTrajectory {
    pub(crate) fn new(
        states: Vec<Identifier>,
        parameters: Vec<Identifier>,
        times: Vec<f64>,
        state: Vec<Vec<f64>>,
        sensitivities: Vec<Vec<Vec<f64>>>,
    ) -> Self {
        Self { states, parameters, times, state, sensitivities }
    }

    /// The state ordering that indexes every state and sensitivity vector.
    pub fn states(&self) -> &[Identifier] {
        &self.states
    }

    /// The parameter ordering that indexes the sensitivity blocks.
    pub fn parameters(&self) -> &[Identifier] {
        &self.parameters
    }

    /// The state-space dimension `n`.
    pub fn dimension(&self) -> usize {
        self.states.len()
    }

    /// The number of parameters `p` whose sensitivities were integrated.
    pub fn parameter_count(&self) -> usize {
        self.parameters.len()
    }

    /// The shared time grid, `steps + 1` instants beginning at `t0`.
    pub fn times(&self) -> &[f64] {
        &self.times
    }

    /// The number of time samples (`steps + 1`).
    pub fn sample_count(&self) -> usize {
        self.times.len()
    }

    /// The state vector `x(times[step])` in `states` order, or `None` if `step`
    /// is out of range.
    pub fn state_at(&self, step: usize) -> Option<&[f64]> {
        self.state.get(step).map(Vec::as_slice)
    }

    /// The sensitivity vector `S_j(times[step]) = ∂x(times[step])/∂θ_j` in
    /// `states` order, or `None` if either index is out of range.
    pub fn sensitivity_at(&self, parameter: usize, step: usize) -> Option<&[f64]> {
        self.sensitivities.get(parameter).and_then(|block| block.get(step)).map(Vec::as_slice)
    }

    /// The scalar sensitivity `∂x_i(times[step])/∂θ_j` — how the `state`-th
    /// component of the forecast at the `step`-th instant responds to a change in
    /// the `parameter`-th coefficient. `None` if any index is out of range.
    pub fn partial(&self, state: usize, parameter: usize, step: usize) -> Option<f64> {
        self.sensitivities
            .get(parameter)
            .and_then(|block| block.get(step))
            .and_then(|vector| vector.get(state))
            .copied()
    }

    /// A stable textual fingerprint of the whole trajectory, encoding every float
    /// by its `f64` bit pattern. Two runs on identical input MUST produce
    /// identical strings; this is the basis of the determinism guarantee.
    pub fn to_canonical_string(&self) -> String {
        let mut output = String::new();
        output.push_str("states:");
        for state in &self.states {
            output.push_str(state.as_str());
            output.push(',');
        }
        output.push_str("\nparameters:");
        for parameter in &self.parameters {
            output.push_str(parameter.as_str());
            output.push(',');
        }
        output.push('\n');
        for (step, time) in self.times.iter().enumerate() {
            let _ = write!(output, "t={:016x} x:", time.to_bits());
            for value in &self.state[step] {
                let _ = write!(output, "{:016x},", value.to_bits());
            }
            for (parameter, block) in self.sensitivities.iter().enumerate() {
                let _ = write!(output, " S{parameter}:");
                for value in &block[step] {
                    let _ = write!(output, "{:016x},", value.to_bits());
                }
            }
            output.push('\n');
        }
        output
    }
}
