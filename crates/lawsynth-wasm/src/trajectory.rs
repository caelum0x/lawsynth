use crate::WasmError;
#[derive(Clone, Debug, PartialEq)]
pub struct Trajectory {
    pub times: Vec<f64>,
    pub values: Vec<Vec<f64>>,
}
impl Trajectory {
    pub fn new(times: Vec<f64>, values: Vec<Vec<f64>>) -> Result<Self, WasmError> {
        if times.is_empty() || times.len() != values.len() {
            return Err(WasmError::InvalidTrajectory(
                "times and rows must have matching nonzero length".into(),
            ));
        }
        let width = values[0].len();
        if width == 0
            || times.iter().any(|value| !value.is_finite())
            || values
                .iter()
                .any(|row| row.len() != width || row.iter().any(|value| !value.is_finite()))
            || times.windows(2).any(|pair| pair[1] <= pair[0])
        {
            return Err(WasmError::InvalidTrajectory(
                "trajectory must be finite, rectangular, and strictly increasing".into(),
            ));
        }
        Ok(Self { times, values })
    }
    pub fn dimension(&self) -> usize {
        self.values[0].len()
    }
    pub fn len(&self) -> usize {
        self.times.len()
    }
    pub fn is_empty(&self) -> bool {
        false
    }
}
