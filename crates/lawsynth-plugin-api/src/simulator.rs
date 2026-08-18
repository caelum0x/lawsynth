use crate::{PluginError, ResourceLimits};

const DEFAULT_MAX_TIME_POINTS: usize = 1_000_000;

/// State vector and absolute, strictly increasing output times for a simulator
/// plugin invocation.
#[derive(Clone, Debug, PartialEq)]
pub struct SimulationRequest {
    pub initial_state: Vec<f64>,
    pub times: Vec<f64>,
}

impl SimulationRequest {
    pub fn validate(&self) -> Result<(), PluginError> {
        self.validate_with_max_points(DEFAULT_MAX_TIME_POINTS)
    }

    pub fn validate_with_limits(&self, limits: ResourceLimits) -> Result<(), PluginError> {
        limits.validate()?;
        let bytes_per_state = self
            .initial_state
            .len()
            .checked_mul(size_of::<f64>())
            .ok_or_else(|| PluginError::ResourceLimit("state size overflow".into()))?;
        let max_by_memory = if bytes_per_state == 0 {
            0
        } else {
            usize::try_from(limits.max_memory_bytes)
                .unwrap_or(usize::MAX)
                .checked_div(bytes_per_state)
                .unwrap_or(0)
        };
        self.validate_with_max_points(max_by_memory.min(limits.max_requests as usize))
    }

    pub fn validate_with_max_points(&self, max_points: usize) -> Result<(), PluginError> {
        if self.initial_state.is_empty() {
            return Err(PluginError::InvalidData(
                "simulation initial state must not be empty".into(),
            ));
        }
        if self.initial_state.iter().any(|value| !value.is_finite()) {
            return Err(PluginError::InvalidData("simulation initial state must be finite".into()));
        }
        if self.times.is_empty() {
            return Err(PluginError::InvalidData("simulation times must not be empty".into()));
        }
        if self.times.len() > max_points {
            return Err(PluginError::ResourceLimit(format!(
                "simulation requests {} time points, limit is {max_points}",
                self.times.len()
            )));
        }
        if self.times.iter().any(|time| !time.is_finite()) {
            return Err(PluginError::InvalidData("simulation times must be finite".into()));
        }
        if self.times.windows(2).any(|window| window[1] <= window[0]) {
            return Err(PluginError::InvalidData(
                "simulation times must be strictly increasing".into(),
            ));
        }
        Ok(())
    }

    pub fn estimated_output_bytes(&self) -> Result<usize, PluginError> {
        self.times
            .len()
            .checked_mul(self.initial_state.len())
            .and_then(|values| values.checked_mul(size_of::<f64>()))
            .ok_or_else(|| PluginError::ResourceLimit("simulation output size overflow".into()))
    }
}

/// Dense state matrix in time-major order (`states[time][state]`).
#[derive(Clone, Debug, PartialEq)]
pub struct SimulationResponse {
    pub states: Vec<Vec<f64>>,
}

impl SimulationResponse {
    pub fn validate_for(&self, request: &SimulationRequest) -> Result<(), PluginError> {
        if self.states.len() != request.times.len() {
            return Err(PluginError::InvalidData(format!(
                "simulation returned {} states for {} requested times",
                self.states.len(),
                request.times.len()
            )));
        }
        for (index, state) in self.states.iter().enumerate() {
            if state.len() != request.initial_state.len() {
                return Err(PluginError::InvalidData(format!(
                    "state {index} has width {}, expected {}",
                    state.len(),
                    request.initial_state.len()
                )));
            }
            if state.iter().any(|value| !value.is_finite()) {
                return Err(PluginError::InvalidData(format!(
                    "state {index} contains a non-finite value"
                )));
            }
        }
        Ok(())
    }

    pub fn validate_for_with_limits(
        &self,
        request: &SimulationRequest,
        limits: ResourceLimits,
    ) -> Result<(), PluginError> {
        limits.validate()?;
        self.validate_for(request)?;
        let output_bytes = request.estimated_output_bytes()?;
        if output_bytes > usize::try_from(limits.max_output_bytes).unwrap_or(usize::MAX) {
            return Err(PluginError::ResourceLimit(format!(
                "simulation output size {output_bytes} exceeds limit {}",
                limits.max_output_bytes
            )));
        }
        Ok(())
    }
}

pub trait SimulationPlugin: Send + Sync {
    fn simulate(&self, request: SimulationRequest) -> Result<SimulationResponse, PluginError>;
}
