//! Configuration for a receding-horizon control run.

use lawsynth_koopman::Matrix;

/// Everything the controller needs beyond the model itself: the regulation
/// target, the LQR weights, the integration step and horizon, the initial
/// condition, and optional control saturation.
///
/// Build one with [`MpcConfig::new`] (which defaults the control reference to
/// zero and leaves the actuator unsaturated) and refine it with the
/// value-returning setters [`MpcConfig::with_control_reference`] and
/// [`MpcConfig::with_saturation`]. The struct is validated at the boundary by
/// [`mpc_control`](crate::mpc_control) against the actual model dimensions, so a
/// mis-sized weight or setpoint surfaces as a typed
/// [`MpcError`](crate::MpcError) rather than a panic.
#[derive(Clone, Debug)]
pub struct MpcConfig {
    /// Initial plant state `x₀`, length `n` (state dimension).
    pub initial_state: Vec<f64>,
    /// Regulation setpoint `x_ref`, length `n`.
    pub setpoint: Vec<f64>,
    /// Control reference `u_ref`, length `m` (control dimension). The applied
    /// law is `u = u_ref − K (x − x_ref)`. Defaults to zeros.
    pub control_reference: Vec<f64>,
    /// LQR state weight `Q`, shape `n × n` (symmetric positive semidefinite).
    pub state_weight: Matrix,
    /// LQR control weight `R`, shape `m × m` (symmetric positive definite).
    pub control_weight: Matrix,
    /// Fixed integration step `dt` for the RK4 plant advance (seconds).
    pub dt: f64,
    /// Number of closed-loop control steps to simulate.
    pub steps: usize,
    /// Optional lower saturation bound per control channel, length `m`.
    pub control_min: Option<Vec<f64>>,
    /// Optional upper saturation bound per control channel, length `m`.
    pub control_max: Option<Vec<f64>>,
}

impl MpcConfig {
    /// Creates a configuration with a zero control reference and no saturation.
    ///
    /// The control dimension `m` is read from `control_weight` (an `m × m`
    /// matrix), so the default `control_reference` is `vec![0.0; m]`.
    pub fn new(
        initial_state: Vec<f64>,
        setpoint: Vec<f64>,
        state_weight: Matrix,
        control_weight: Matrix,
        dt: f64,
        steps: usize,
    ) -> Self {
        let m = control_weight.rows();
        Self {
            initial_state,
            setpoint,
            control_reference: vec![0.0; m],
            state_weight,
            control_weight,
            dt,
            steps,
            control_min: None,
            control_max: None,
        }
    }

    /// Returns a copy with the control reference `u_ref` replaced.
    pub fn with_control_reference(self, control_reference: Vec<f64>) -> Self {
        Self { control_reference, ..self }
    }

    /// Returns a copy with element-wise control saturation `[u_min, u_max]`.
    pub fn with_saturation(self, control_min: Vec<f64>, control_max: Vec<f64>) -> Self {
        Self { control_min: Some(control_min), control_max: Some(control_max), ..self }
    }
}
