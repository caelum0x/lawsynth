use crate::{RegimeError, Result};
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub start: usize,
    pub end: usize,
    pub mean: f64,
    pub sum_squared_error: f64,
}
impl Segment {
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Segmentation {
    pub segments: Vec<Segment>,
    pub objective: f64,
}
impl Segmentation {
    pub fn new(segments: Vec<Segment>, objective: f64, observations: usize) -> Result<Self> {
        if segments.is_empty() || !objective.is_finite() {
            return Err(RegimeError::InvalidParameter("segmentation"));
        }
        let mut expected = 0;
        for s in &segments {
            if s.start != expected
                || s.end <= s.start
                || s.end > observations
                || !s.mean.is_finite()
                || s.sum_squared_error < 0.0
            {
                return Err(RegimeError::InvalidSegment { start: s.start, end: s.end });
            }
            expected = s.end;
        }
        if expected != observations {
            return Err(RegimeError::InvalidSegment { start: expected, end: observations });
        }
        Ok(Self { segments, objective })
    }
    pub fn change_points(&self) -> Vec<usize> {
        self.segments.iter().take(self.segments.len().saturating_sub(1)).map(|s| s.end).collect()
    }
    pub fn label_at(&self, index: usize) -> Option<usize> {
        self.segments.iter().position(|s| s.start <= index && index < s.end)
    }
}
