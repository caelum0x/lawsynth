//! The closed-loop result of a control run.

use lawsynth_koopman::Matrix;

/// The recorded closed-loop trajectory of a receding-horizon run.
///
/// - `states` holds `steps + 1` state vectors: the initial state followed by
///   the state after each RK4 advance.
/// - `controls` holds the `steps` applied control moves (one per step, held
///   across that step).
/// - `gains` holds the `steps` local LQR gains `K` designed at each step's
///   linearization point (exposed so callers can inspect gain scheduling and
///   check linear consistency).
/// - `times` holds the `steps + 1` sample times `0, dt, 2·dt, …`.
///
/// All vectors follow the caller's state/control ordering. The trajectory is a
/// pure function of the inputs, so two runs with identical inputs produce
/// bit-identical `states` and `controls` (compare via [`f64::to_bits`]).
#[derive(Clone, Debug)]
pub struct MpcTrajectory {
    states: Vec<Vec<f64>>,
    controls: Vec<Vec<f64>>,
    gains: Vec<Matrix>,
    times: Vec<f64>,
}

impl MpcTrajectory {
    pub(crate) fn new(
        states: Vec<Vec<f64>>,
        controls: Vec<Vec<f64>>,
        gains: Vec<Matrix>,
        times: Vec<f64>,
    ) -> Self {
        Self { states, controls, gains, times }
    }

    /// The state trajectory, `steps + 1` vectors including the initial state.
    pub fn states(&self) -> &[Vec<f64>] {
        &self.states
    }

    /// The applied control moves, one per step.
    pub fn controls(&self) -> &[Vec<f64>] {
        &self.controls
    }

    /// The per-step local LQR gains `K`.
    pub fn gains(&self) -> &[Matrix] {
        &self.gains
    }

    /// The sample times aligned with [`states`](Self::states).
    pub fn times(&self) -> &[f64] {
        &self.times
    }

    /// The final closed-loop state.
    pub fn final_state(&self) -> &[f64] {
        // `states` always holds at least the initial state, so this never panics.
        self.states.last().map(Vec::as_slice).unwrap_or(&[])
    }

    /// The Euclidean distance `‖x_k − x_ref‖₂` of the state at step `k` from a
    /// setpoint. Returns `None` if `k` is out of range or the lengths differ.
    pub fn error_norm(&self, step: usize, setpoint: &[f64]) -> Option<f64> {
        let state = self.states.get(step)?;
        if state.len() != setpoint.len() {
            return None;
        }
        let sum_squares: f64 =
            state.iter().zip(setpoint).map(|(value, target)| (value - target).powi(2)).sum();
        Some(sum_squares.sqrt())
    }

    /// The final-state error norm relative to a setpoint.
    pub fn final_error_norm(&self, setpoint: &[f64]) -> Option<f64> {
        self.error_norm(self.states.len().saturating_sub(1), setpoint)
    }

    /// A deterministic fingerprint of the whole trajectory as raw `f64` bit
    /// patterns (states then controls, in order). Two runs are bit-identical iff
    /// their fingerprints are equal.
    pub fn bit_fingerprint(&self) -> Vec<u64> {
        let mut bits = Vec::new();
        for state in &self.states {
            bits.extend(state.iter().map(|value| value.to_bits()));
        }
        for control in &self.controls {
            bits.extend(control.iter().map(|value| value.to_bits()));
        }
        bits
    }
}
