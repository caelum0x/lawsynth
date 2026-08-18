use crate::{RegimeError, Result};
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SegmentMoments {
    pub count: usize,
    pub sum: f64,
    pub sum_squares: f64,
    pub mean: f64,
    pub sum_squared_error: f64,
}
pub fn segment_moments(data: &[f64], start: usize, end: usize) -> Result<SegmentMoments> {
    if start >= end || end > data.len() {
        return Err(RegimeError::InvalidSegment { start, end });
    }
    let values = &data[start..end];
    for (offset, &v) in values.iter().enumerate() {
        if !v.is_finite() {
            return Err(RegimeError::NonFiniteObservation { index: start + offset });
        }
    }
    let count = values.len();
    let sum: f64 = values.iter().sum();
    let sum_squares: f64 = values.iter().map(|v| v * v).sum();
    let mean = sum / count as f64;
    let sse = (sum_squares - sum * sum / count as f64).max(0.0);
    Ok(SegmentMoments { count, sum, sum_squares, mean, sum_squared_error: sse })
}
pub fn segment_cost(data: &[f64], start: usize, end: usize) -> Result<f64> {
    Ok(segment_moments(data, start, end)?.sum_squared_error)
}
