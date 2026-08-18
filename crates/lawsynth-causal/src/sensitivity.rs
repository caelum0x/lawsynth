use crate::{CausalError, Result};
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfoundingBound {
    pub observed_risk_ratio: f64,
    pub e_value: f64,
}
pub fn e_value(risk_ratio: f64) -> Result<ConfoundingBound> {
    if !risk_ratio.is_finite() || risk_ratio <= 0.0 {
        return Err(CausalError::InvalidParameter("risk_ratio"));
    }
    let rr = if risk_ratio < 1.0 { 1.0 / risk_ratio } else { risk_ratio };
    let value = if rr == 1.0 { 1.0 } else { rr + (rr * (rr - 1.0)).sqrt() };
    Ok(ConfoundingBound { observed_risk_ratio: risk_ratio, e_value: value })
}
