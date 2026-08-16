use lawsynth_core::Identifier;

/// A named scalar measurement column aligned with a Dataset time axis.
#[derive(Clone, Debug, PartialEq)]
pub struct NumericColumn {
    pub id: Identifier,
    pub values: Vec<f64>,
    pub unit: Option<String>,
}

impl NumericColumn {
    pub fn new(id: Identifier, values: Vec<f64>) -> Self {
        Self { id, values, unit: None }
    }

    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }
}
