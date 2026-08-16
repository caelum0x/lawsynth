use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_world::{DiscreteWorld, World};

/// Immutable law plan compiled from a continuous World before integration.
#[derive(Clone, Debug, PartialEq)]
pub struct CompiledContinuousWorld {
    laws: Vec<(Identifier, Expr)>,
}

impl CompiledContinuousWorld {
    pub fn compile(world: &World) -> Self {
        Self {
            laws: world
                .laws()
                .iter()
                .map(|(id, law)| (id.clone(), law.expression.clone()))
                .collect(),
        }
    }

    pub(crate) fn laws(&self) -> &[(Identifier, Expr)] {
        &self.laws
    }

    pub fn law_targets(&self) -> impl Iterator<Item = &Identifier> {
        self.laws.iter().map(|(target, _)| target)
    }
}

/// Immutable simultaneous-update law plan compiled from a discrete World.
#[derive(Clone, Debug, PartialEq)]
pub struct CompiledDiscreteWorld {
    laws: Vec<(Identifier, Expr)>,
}

impl CompiledDiscreteWorld {
    pub fn compile(world: &DiscreteWorld) -> Self {
        Self {
            laws: world
                .laws()
                .iter()
                .map(|(id, law)| (id.clone(), law.expression.clone()))
                .collect(),
        }
    }

    pub(crate) fn laws(&self) -> &[(Identifier, Expr)] {
        &self.laws
    }

    pub fn law_targets(&self) -> impl Iterator<Item = &Identifier> {
        self.laws.iter().map(|(target, _)| target)
    }
}
