use crate::{CausalError, Result};
#[derive(Debug, Clone, PartialEq)]
pub struct TimeOrder {
    pub start: f64,
    pub end: f64,
    pub observations: usize,
}
pub fn validate_time_order(times: &[f64]) -> Result<TimeOrder> {
    if times.is_empty() {
        return Err(CausalError::EmptySeries);
    }
    for (i, pair) in times.windows(2).enumerate() {
        if !pair[0].is_finite() || !pair[1].is_finite() || pair[1] <= pair[0] {
            return Err(CausalError::NonMonotonicTime { index: i + 1 });
        }
    }
    if !times[0].is_finite() {
        return Err(CausalError::NonMonotonicTime { index: 0 });
    }
    Ok(TimeOrder { start: times[0], end: times[times.len() - 1], observations: times.len() })
}
