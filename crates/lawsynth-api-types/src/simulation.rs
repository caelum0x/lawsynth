use crate::{ApiValidationError, WorldRevision};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TimeRange {
    pub start: f64,
    pub end: f64,
    pub step: f64,
}

impl TimeRange {
    pub fn new(start: f64, end: f64, step: f64) -> Result<Self, ApiValidationError> {
        if !start.is_finite() || !end.is_finite() || !step.is_finite() {
            return Err(ApiValidationError::Invalid {
                field: "time_range",
                reason: "values must be finite",
            });
        }
        if end <= start {
            return Err(ApiValidationError::Invalid {
                field: "end",
                reason: "must be greater than start",
            });
        }
        if step <= 0.0 {
            return Err(ApiValidationError::Invalid { field: "step", reason: "must be positive" });
        }
        let samples = ((end - start) / step).ceil();
        if samples > 10_000_000.0 {
            return Err(ApiValidationError::Invalid {
                field: "time_range",
                reason: "exceeds ten million samples",
            });
        }
        Ok(Self { start, end, step })
    }
    pub fn sample_count(self) -> u64 {
        ((self.end - self.start) / self.step).ceil() as u64 + 1
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SimulationRequest {
    pub world: WorldRevision,
    pub time: TimeRange,
    pub seed: u64,
    pub output_variables: Vec<String>,
}

impl SimulationRequest {
    pub fn new(
        world: WorldRevision,
        time: TimeRange,
        seed: u64,
        output_variables: Vec<String>,
    ) -> Result<Self, ApiValidationError> {
        if output_variables.is_empty() {
            return Err(ApiValidationError::Empty { field: "output_variables" });
        }
        if output_variables.iter().any(|name| name.is_empty() || name.len() > 128) {
            return Err(ApiValidationError::Invalid {
                field: "output_variables",
                reason: "each name must be 1..=128 bytes",
            });
        }
        for (index, name) in output_variables.iter().enumerate() {
            if output_variables[..index].iter().any(|prior| prior == name) {
                return Err(ApiValidationError::Invalid {
                    field: "output_variables",
                    reason: "names must be unique",
                });
            }
        }
        Ok(Self { world, time, seed, output_variables })
    }
}
