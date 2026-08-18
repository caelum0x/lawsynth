use crate::{RegimeError, Result, segment_cost};
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BinarySplit {
    pub index: usize,
    pub gain: f64,
    pub left_cost: f64,
    pub right_cost: f64,
}
pub fn best_binary_split(data: &[f64], min_segment_len: usize) -> Result<Option<BinarySplit>> {
    if min_segment_len == 0 {
        return Err(RegimeError::InvalidParameter("min_segment_len"));
    }
    if data.len() < 2 * min_segment_len {
        return Ok(None);
    }
    let full = segment_cost(data, 0, data.len())?;
    let mut best: Option<BinarySplit> = None;
    for index in min_segment_len..=data.len() - min_segment_len {
        let left = segment_cost(data, 0, index)?;
        let right = segment_cost(data, index, data.len())?;
        let candidate =
            BinarySplit { index, gain: full - left - right, left_cost: left, right_cost: right };
        if best.is_none_or(|b| candidate.gain > b.gain) {
            best = Some(candidate);
        }
    }
    Ok(best)
}
