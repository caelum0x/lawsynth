use crate::{RegimeError, Result};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AffineLaw {
    pub intercept: f64,
    pub slope: f64,
}
impl AffineLaw {
    pub fn evaluate(self, x: f64) -> f64 {
        self.intercept + self.slope * x
    }
}
#[derive(Debug, Clone, Default)]
pub struct RegimeLawBook {
    laws: BTreeMap<usize, AffineLaw>,
}
impl RegimeLawBook {
    pub fn insert(&mut self, regime: usize, law: AffineLaw) -> Result<()> {
        if !law.intercept.is_finite() || !law.slope.is_finite() {
            return Err(RegimeError::InvalidParameter("law"));
        }
        self.laws.insert(regime, law);
        Ok(())
    }
    pub fn evaluate(&self, regime: usize, x: f64) -> Result<f64> {
        if !x.is_finite() {
            return Err(RegimeError::NonFiniteObservation { index: 0 });
        }
        self.laws
            .get(&regime)
            .map(|law| law.evaluate(x))
            .ok_or(RegimeError::InvalidParameter("unknown regime"))
    }
    pub fn get(&self, regime: usize) -> Option<AffineLaw> {
        self.laws.get(&regime).copied()
    }
}
