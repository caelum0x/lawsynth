use crate::DataError;

/// Strictly increasing finite numeric timestamps.
#[derive(Clone, Debug, PartialEq)]
pub struct TimeAxis {
    values: Vec<f64>,
}

impl TimeAxis {
    pub fn new(values: Vec<f64>) -> Result<Self, DataError> {
        if values.is_empty() {
            return Err(DataError::EmptyTimeAxis);
        }
        for (index, value) in values.iter().copied().enumerate() {
            if !value.is_finite() {
                return Err(DataError::NonFiniteTimestamp { index, value });
            }
            if index > 0 && value <= values[index - 1] {
                return Err(DataError::NonIncreasingTimestamp { index });
            }
        }
        Ok(Self { values })
    }

    pub fn values(&self) -> &[f64] {
        &self.values
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn is_regular(&self, relative_tolerance: f64) -> bool {
        if self.values.len() < 3 || !relative_tolerance.is_finite() || relative_tolerance < 0.0 {
            return self.values.len() < 3;
        }
        let reference = self.values[1] - self.values[0];
        self.values.windows(2).all(|pair| {
            let interval = pair[1] - pair[0];
            (interval - reference).abs() <= reference.abs().max(1.0) * relative_tolerance
        })
    }
}
