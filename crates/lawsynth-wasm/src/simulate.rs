use crate::{Trajectory, WasmConfig, WasmError, World};
/// Simulate a world using classical fourth-order Runge-Kutta at a fixed time step.
pub fn simulate_rk4(
    world: &World,
    start: f64,
    end: f64,
    step: f64,
    config: &WasmConfig,
) -> Result<Trajectory, WasmError> {
    config.validate()?;
    if !start.is_finite() || !end.is_finite() || !step.is_finite() || step <= 0.0 || end < start {
        return Err(WasmError::Simulation(
            "start, end, and step are invalid".into(),
        ));
    }
    let estimated = ((end - start) / step).ceil() as usize + 1;
    if estimated > config.max_steps {
        return Err(WasmError::Simulation(format!(
            "simulation needs {estimated} steps, limit is {}",
            config.max_steps
        )));
    }
    let mut times = vec![start];
    let mut rows = vec![world.initial_state.clone()];
    let mut time = start;
    let mut state = world.initial_state.clone();
    while time < end {
        let h = (end - time).min(step);
        let k1 = world.derivative_at(time, &state)?;
        let stage = |base: &[f64], derivative: &[f64], scale: f64| -> Vec<f64> {
            base.iter()
                .zip(derivative)
                .map(|(v, d)| v + d * scale)
                .collect()
        };
        let k2 = world.derivative_at(time + h / 2.0, &stage(&state, &k1, h / 2.0))?;
        let k3 = world.derivative_at(time + h / 2.0, &stage(&state, &k2, h / 2.0))?;
        let k4 = world.derivative_at(time + h, &stage(&state, &k3, h))?;
        for index in 0..state.len() {
            state[index] += h * (k1[index] + 2.0 * k2[index] + 2.0 * k3[index] + k4[index]) / 6.0;
        }
        if state.iter().any(|value| !value.is_finite()) {
            return Err(WasmError::Simulation("state became non-finite".into()));
        }
        time += h;
        times.push(time);
        rows.push(state.clone());
    }
    Trajectory::new(times, rows)
}
