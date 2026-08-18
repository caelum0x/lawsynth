use crate::UncertaintyError;

/// Validated one-dimensional observations.
#[derive(Clone, Debug, PartialEq)]
pub struct Samples {
    values: Vec<f64>,
}

impl Samples {
    pub fn new(values: Vec<f64>) -> Result<Self, UncertaintyError> {
        if values.is_empty() {
            return Err(UncertaintyError::EmptyInput);
        }
        if values.iter().any(|value| !value.is_finite()) {
            return Err(UncertaintyError::NonFiniteValue);
        }
        Ok(Self { values })
    }

    pub fn as_slice(&self) -> &[f64] {
        &self.values
    }
    pub fn len(&self) -> usize {
        self.values.len()
    }
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    pub fn mean(&self) -> f64 {
        self.values.iter().sum::<f64>() / self.values.len() as f64
    }

    /// Unbiased sample variance.
    pub fn variance(&self) -> Result<f64, UncertaintyError> {
        if self.len() < 2 {
            return Err(UncertaintyError::TooFewSamples { minimum: 2, actual: self.len() });
        }
        let mean = self.mean();
        Ok(self.values.iter().map(|value| (value - mean).powi(2)).sum::<f64>()
            / (self.len() - 1) as f64)
    }

    pub fn standard_error(&self) -> Result<f64, UncertaintyError> {
        Ok((self.variance()? / self.len() as f64).sqrt())
    }
}

impl TryFrom<Vec<f64>> for Samples {
    type Error = UncertaintyError;
    fn try_from(values: Vec<f64>) -> Result<Self, Self::Error> {
        Self::new(values)
    }
}
