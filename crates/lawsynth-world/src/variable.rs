use lawsynth_core::Identifier;
use lawsynth_units::Unit;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VariableRole {
    State,
    Control,
    Exogenous,
    Observed,
    Latent,
    Derived,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    pub id: Identifier,
    pub role: VariableRole,
    pub unit: Option<Unit>,
}

impl Variable {
    pub fn new(id: Identifier, role: VariableRole) -> Self {
        Self { id, role, unit: None }
    }

    pub fn with_unit(mut self, unit: Unit) -> Self {
        self.unit = Some(unit);
        self
    }
}
