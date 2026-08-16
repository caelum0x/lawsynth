use lawsynth_plugin_api::{PluginError, SimulationPlugin, SimulationRequest, SimulationResponse};

#[derive(Clone, Debug)]
pub struct LinearSimulator {
    matrix: Vec<Vec<f64>>,
    bias: Vec<f64>,
    max_step: f64,
}

impl LinearSimulator {
    pub fn new(matrix: Vec<Vec<f64>>, bias: Vec<f64>, max_step: f64) -> Result<Self, PluginError> {
        let width = bias.len();
        if width == 0 || matrix.len() != width || matrix.iter().any(|row| row.len() != width) {
            return Err(PluginError::InvalidData("linear simulator matrix must be square".into()));
        }
        if !max_step.is_finite() || max_step <= 0.0 || matrix.iter().flatten().chain(&bias).any(|x| !x.is_finite()) {
            return Err(PluginError::InvalidData("simulator parameters must be finite".into()));
        }
        Ok(Self { matrix, bias, max_step })
    }

    fn derivative(&self, state: &[f64]) -> Vec<f64> {
        self.matrix.iter().zip(&self.bias).map(|(row, bias)|
            row.iter().zip(state).map(|(a, x)| a * x).sum::<f64>() + bias
        ).collect()
    }
}

impl SimulationPlugin for LinearSimulator {
    fn simulate(&self, request: SimulationRequest) -> Result<SimulationResponse, PluginError> {
        request.validate()?;
        if request.initial_state.len() != self.bias.len() {
            return Err(PluginError::InvalidData("initial state width does not match simulator".into()));
        }
        let mut state = request.initial_state.clone();
        let mut states = Vec::with_capacity(request.times.len());
        let mut time = request.times[0];
        states.push(state.clone());
        for &target in request.times.iter().skip(1) {
            while time < target {
                let step = self.max_step.min(target - time);
                let derivative = self.derivative(&state);
                for (value, change) in state.iter_mut().zip(derivative) { *value += step * change; }
                if state.iter().any(|x| !x.is_finite()) {
                    return Err(PluginError::InvalidData("simulation diverged to a non-finite state".into()));
                }
                time += step;
            }
            states.push(state.clone());
        }
        let response = SimulationResponse { states };
        response.validate_for(&request)?;
        Ok(response)
    }
}
