use crate::OptimizationError;

/// Inclusive scalar bounds applied uniformly to every optimized constant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParameterBounds {
    pub lower: f64,
    pub upper: f64,
}

impl ParameterBounds {
    pub fn new(lower: f64, upper: f64) -> Result<Self, OptimizationError> {
        if !lower.is_finite() || !upper.is_finite() || lower > upper {
            return Err(OptimizationError::InvalidBounds);
        }
        Ok(Self { lower, upper })
    }

    pub fn clamp(self, value: f64) -> f64 {
        value.clamp(self.lower, self.upper)
    }
}
