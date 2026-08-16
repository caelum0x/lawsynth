use lawsynth_core::Identifier;
use lawsynth_units::Unit;

/// A scalar parameter held constant during one simulation.
#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    pub id: Identifier,
    pub value: f64,
    pub unit: Option<Unit>,
}

impl Parameter {
    pub fn new(id: Identifier, value: f64) -> Self {
        Self {
            id,
            value,
            unit: None,
        }
    }

    pub fn with_unit(mut self, unit: Unit) -> Self {
        self.unit = Some(unit);
        self
    }
}
