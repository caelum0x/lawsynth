use crate::{Expression, WasmError};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventDirection {
    Any,
    Rising,
    Falling,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub name: String,
    pub condition: Expression,
    pub direction: EventDirection,
}
#[derive(Clone, Debug, PartialEq)]
pub struct EventOccurrence {
    pub name: String,
    pub time: f64,
    pub value: f64,
}
impl Event {
    pub fn new(
        name: impl Into<String>,
        condition: Expression,
        direction: EventDirection,
    ) -> Result<Self, WasmError> {
        let name = name.into();
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(WasmError::InvalidWorld(
                "event name must be an identifier".into(),
            ));
        }
        Ok(Self {
            name,
            condition,
            direction,
        })
    }
    pub fn crosses(&self, before: f64, after: f64) -> bool {
        match self.direction {
            EventDirection::Any => {
                (before <= 0.0 && after >= 0.0) || (before >= 0.0 && after <= 0.0)
            }
            EventDirection::Rising => before < 0.0 && after >= 0.0,
            EventDirection::Falling => before > 0.0 && after <= 0.0,
        }
    }
    pub fn evaluate(&self, values: &BTreeMap<String, f64>) -> Result<f64, WasmError> {
        self.condition.evaluate(values)
    }
}
