use crate::{CausalError, Result};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CausalConfig {
    pub max_lag: usize,
    pub min_samples: usize,
    pub singular_tolerance: f64,
}

impl Default for CausalConfig {
    fn default() -> Self {
        Self { max_lag: 1, min_samples: 12, singular_tolerance: 1e-12 }
    }
}

impl CausalConfig {
    pub fn validate(self) -> Result<Self> {
        if self.max_lag == 0 {
            return Err(CausalError::InvalidParameter("max_lag"));
        }
        if self.min_samples <= 2 * self.max_lag + 2 {
            return Err(CausalError::InvalidParameter("min_samples"));
        }
        if !self.singular_tolerance.is_finite() || self.singular_tolerance <= 0.0 {
            return Err(CausalError::InvalidParameter("singular_tolerance"));
        }
        Ok(self)
    }
}
