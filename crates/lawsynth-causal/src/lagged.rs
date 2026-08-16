use crate::{CausalError, Result};
#[derive(Debug, Clone, PartialEq)]
pub struct LaggedObservation {
    pub index: usize,
    pub target: f64,
    pub history: Vec<f64>,
}
pub fn lagged_observations(series: &[f64], lag: usize) -> Result<Vec<LaggedObservation>> {
    if lag == 0 {
        return Err(CausalError::InvalidParameter("lag"));
    }
    if series.len() <= lag {
        return Err(CausalError::InsufficientSamples {
            required: lag + 1,
            actual: series.len(),
        });
    }
    if series.iter().any(|v| !v.is_finite()) {
        return Err(CausalError::InvalidParameter("series"));
    }
    Ok((lag..series.len())
        .map(|i| LaggedObservation {
            index: i,
            target: series[i],
            history: (1..=lag).map(|j| series[i - j]).collect(),
        })
        .collect())
}
