use crate::{CausalError, Result};
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IndependenceResult {
    pub correlation: f64,
    pub sample_size: usize,
}
impl IndependenceResult {
    pub fn is_near_independent(self, tolerance: f64) -> bool {
        self.correlation.abs() <= tolerance
    }
}
pub fn pearson_independence(x: &[f64], y: &[f64]) -> Result<IndependenceResult> {
    if x.len() != y.len() {
        return Err(CausalError::LengthMismatch {
            expected: x.len(),
            actual: y.len(),
        });
    }
    if x.len() < 2 {
        return Err(CausalError::InsufficientSamples {
            required: 2,
            actual: x.len(),
        });
    }
    let (mx, my) = (
        x.iter().sum::<f64>() / x.len() as f64,
        y.iter().sum::<f64>() / y.len() as f64,
    );
    let (mut xy, mut xx, mut yy) = (0.0, 0.0, 0.0);
    for (&a, &b) in x.iter().zip(y) {
        if !a.is_finite() || !b.is_finite() {
            return Err(CausalError::InvalidParameter("series"));
        }
        let (da, db) = (a - mx, b - my);
        xy += da * db;
        xx += da * da;
        yy += db * db;
    }
    if xx == 0.0 || yy == 0.0 {
        return Err(CausalError::SingularDesign);
    }
    Ok(IndependenceResult {
        correlation: xy / (xx * yy).sqrt(),
        sample_size: x.len(),
    })
}
