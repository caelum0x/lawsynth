use lawsynth_core::Identifier;

/// The mutable value addressed by a scenario intervention.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InterventionTarget {
    Parameter(Identifier),
    Input(Identifier),
}

impl InterventionTarget {
    pub fn id(&self) -> &Identifier {
        match self {
            Self::Parameter(id) | Self::Input(id) => id,
        }
    }
}

/// A scheduled scenario-level value change for an executable World.
#[derive(Clone, Debug, PartialEq)]
pub struct Intervention {
    pub time: f64,
    pub target: InterventionTarget,
    pub value: f64,
}

impl Intervention {
    pub fn parameter(time: f64, id: Identifier, value: f64) -> Self {
        Self { time, target: InterventionTarget::Parameter(id), value }
    }

    pub fn input(time: f64, id: Identifier, value: f64) -> Self {
        Self { time, target: InterventionTarget::Input(id), value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_intervention_targets() {
        let id = Identifier::new("rate").unwrap();
        let intervention = Intervention::parameter(2.0, id.clone(), 3.0);
        assert_eq!(intervention.target.id(), &id);
    }
}
