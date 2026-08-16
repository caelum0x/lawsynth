use crate::{RegimeError, Result};
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentationConfig {
    pub penalty: f64,
    pub min_segment_len: usize,
}
impl Default for SegmentationConfig {
    fn default() -> Self {
        Self {
            penalty: 8.0,
            min_segment_len: 3,
        }
    }
}
impl SegmentationConfig {
    pub fn validate(self) -> Result<Self> {
        if !self.penalty.is_finite() || self.penalty < 0.0 {
            return Err(RegimeError::InvalidParameter("penalty"));
        }
        if self.min_segment_len == 0 {
            return Err(RegimeError::InvalidParameter("min_segment_len"));
        }
        Ok(self)
    }
}
