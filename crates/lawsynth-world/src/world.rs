use std::collections::{BTreeMap, BTreeSet};

use lawsynth_core::Identifier;
use lawsynth_units::{Dimension, Unit, infer_expression_dimension};

use crate::{
    ContinuousLaw, DiscreteLaw, Parameter, Variable, VariableRole, WorldConfig, WorldError,
    expression_symbols,
};

/// An executable continuous-time world with explicit variables, parameters and
/// one state-transition law for each state variable.
#[derive(Clone, Debug, PartialEq)]
pub struct World {
    variables: BTreeMap<Identifier, Variable>,
    parameters: BTreeMap<Identifier, Parameter>,
    laws: BTreeMap<Identifier, ContinuousLaw>,
}

/// An executable discrete-time world with simultaneous state updates.
#[derive(Clone, Debug, PartialEq)]
pub struct DiscreteWorld {
    variables: BTreeMap<Identifier, Variable>,
    parameters: BTreeMap<Identifier, Parameter>,
    laws: BTreeMap<Identifier, DiscreteLaw>,
}

impl DiscreteWorld {
    pub fn new(
        variables: impl IntoIterator<Item = Variable>,
        parameters: impl IntoIterator<Item = Parameter>,
        laws: impl IntoIterator<Item = DiscreteLaw>,
    ) -> Result<Self, WorldError> {
        Self::new_with_config(variables, parameters, laws, WorldConfig::default())
    }

    pub fn new_with_config(
        variables: impl IntoIterator<Item = Variable>,
        parameters: impl IntoIterator<Item = Parameter>,
        laws: impl IntoIterator<Item = DiscreteLaw>,
        config: WorldConfig,
    ) -> Result<Self, WorldError> {
        let mut variable_map = BTreeMap::new();
        for variable in variables {
            if variable_map.insert(variable.id.clone(), variable.clone()).is_some() {
                return Err(WorldError::DuplicateVariable(variable.id));
            }
        }
        let mut parameter_map = BTreeMap::new();
        for parameter in parameters {
            if variable_map.contains_key(&parameter.id) {
                return Err(WorldError::ParameterConflictsWithVariable(parameter.id));
            }
            if !parameter.value.is_finite() {
                return Err(WorldError::NonFiniteParameter(parameter.id));
            }
            if parameter_map.insert(parameter.id.clone(), parameter.clone()).is_some() {
                return Err(WorldError::DuplicateParameter(parameter.id));
            }
        }
        let mut law_map = BTreeMap::new();
        for law in laws {
            match variable_map.get(&law.target) {
                Some(variable) if variable.role == VariableRole::State => {}
                _ => return Err(WorldError::LawTargetsNonState(law.target)),
            }
            if law_map.insert(law.target.clone(), law.clone()).is_some() {
                return Err(WorldError::DuplicateLaw(law.target));
            }
        }
        for state_id in variable_map
            .values()
            .filter(|variable| variable.role == VariableRole::State)
            .map(|variable| variable.id.clone())
        {
            if !law_map.contains_key(&state_id) {
                return Err(WorldError::StateVariableWithoutLaw(state_id));
            }
        }
        if config.validate_expression_symbols {
            let symbols =
                variable_map.keys().chain(parameter_map.keys()).cloned().collect::<BTreeSet<_>>();
            for law in law_map.values() {
                if let Some(symbol) = expression_symbols(&law.expression)
                    .into_iter()
                    .find(|symbol| !symbols.contains(symbol))
                {
                    return Err(WorldError::UnknownSymbol(symbol));
                }
            }
        }
        let units = variable_map
            .values()
            .filter_map(|variable| {
                variable.unit.as_ref().map(|unit| (variable.id.clone(), unit.clone()))
            })
            .chain(parameter_map.values().filter_map(|parameter| {
                parameter.unit.as_ref().map(|unit| (parameter.id.clone(), unit.clone()))
            }))
            .collect::<BTreeMap<_, Unit>>();
        for law in law_map.values().filter(|_| config.validate_units) {
            let Some(target_unit) = variable_map[&law.target].unit.as_ref() else {
                continue;
            };
            if infer_expression_dimension(&law.expression, &units)? != target_unit.dimension() {
                return Err(WorldError::UnitMismatch(law.target.clone()));
            }
        }
        Ok(Self { variables: variable_map, parameters: parameter_map, laws: law_map })
    }

    pub fn variables(&self) -> &BTreeMap<Identifier, Variable> {
        &self.variables
    }

    pub fn parameters(&self) -> &BTreeMap<Identifier, Parameter> {
        &self.parameters
    }

    pub fn laws(&self) -> &BTreeMap<Identifier, DiscreteLaw> {
        &self.laws
    }

    /// Stable directed dependencies from each updated state to symbols it reads.
    pub fn dependency_graph(&self) -> BTreeMap<Identifier, BTreeSet<Identifier>> {
        self.laws
            .iter()
            .map(|(target, law)| (target.clone(), crate::expression_symbols(&law.expression)))
            .collect()
    }

    pub fn state_ids(&self) -> impl Iterator<Item = &Identifier> {
        self.variables
            .values()
            .filter(|variable| variable.role == VariableRole::State)
            .map(|variable| &variable.id)
    }
}

impl World {
    pub fn new(
        variables: impl IntoIterator<Item = Variable>,
        parameters: impl IntoIterator<Item = Parameter>,
        laws: impl IntoIterator<Item = ContinuousLaw>,
    ) -> Result<Self, WorldError> {
        Self::new_with_config(variables, parameters, laws, WorldConfig::default())
    }

    pub fn new_with_config(
        variables: impl IntoIterator<Item = Variable>,
        parameters: impl IntoIterator<Item = Parameter>,
        laws: impl IntoIterator<Item = ContinuousLaw>,
        config: WorldConfig,
    ) -> Result<Self, WorldError> {
        let mut variable_map = BTreeMap::new();
        for variable in variables {
            if variable_map.insert(variable.id.clone(), variable.clone()).is_some() {
                return Err(WorldError::DuplicateVariable(variable.id));
            }
        }

        let mut parameter_map = BTreeMap::new();
        for parameter in parameters {
            if variable_map.contains_key(&parameter.id) {
                return Err(WorldError::ParameterConflictsWithVariable(parameter.id));
            }
            if !parameter.value.is_finite() {
                return Err(WorldError::NonFiniteParameter(parameter.id));
            }
            if parameter_map.insert(parameter.id.clone(), parameter.clone()).is_some() {
                return Err(WorldError::DuplicateParameter(parameter.id));
            }
        }

        let mut law_map = BTreeMap::new();
        for law in laws {
            match variable_map.get(&law.target) {
                Some(variable) if variable.role == VariableRole::State => {}
                _ => return Err(WorldError::LawTargetsNonState(law.target)),
            }
            if law_map.insert(law.target.clone(), law.clone()).is_some() {
                return Err(WorldError::DuplicateLaw(law.target));
            }
        }

        let state_ids: BTreeSet<_> = variable_map
            .values()
            .filter(|variable| variable.role == VariableRole::State)
            .map(|variable| variable.id.clone())
            .collect();
        for state_id in state_ids {
            if !law_map.contains_key(&state_id) {
                return Err(WorldError::StateVariableWithoutLaw(state_id));
            }
        }

        if config.validate_expression_symbols {
            let symbols =
                variable_map.keys().chain(parameter_map.keys()).cloned().collect::<BTreeSet<_>>();
            for law in law_map.values() {
                if let Some(symbol) = expression_symbols(&law.expression)
                    .into_iter()
                    .find(|symbol| !symbols.contains(symbol))
                {
                    return Err(WorldError::UnknownSymbol(symbol));
                }
            }
        }
        let units = variable_map
            .values()
            .filter_map(|variable| {
                variable.unit.as_ref().map(|unit| (variable.id.clone(), unit.clone()))
            })
            .chain(parameter_map.values().filter_map(|parameter| {
                parameter.unit.as_ref().map(|unit| (parameter.id.clone(), unit.clone()))
            }))
            .collect::<BTreeMap<_, Unit>>();
        for law in law_map.values().filter(|_| config.validate_units) {
            let Some(target_unit) = variable_map[&law.target].unit.as_ref() else {
                continue;
            };
            let expected = target_unit
                .dimension()
                .divide(Dimension::TIME)
                .expect("SI base dimensions cannot overflow when subtracting one");
            let actual = infer_expression_dimension(&law.expression, &units)?;
            if actual != expected {
                return Err(WorldError::UnitMismatch(law.target.clone()));
            }
        }

        Ok(Self { variables: variable_map, parameters: parameter_map, laws: law_map })
    }

    pub fn variables(&self) -> &BTreeMap<Identifier, Variable> {
        &self.variables
    }

    pub fn parameters(&self) -> &BTreeMap<Identifier, Parameter> {
        &self.parameters
    }

    pub fn laws(&self) -> &BTreeMap<Identifier, ContinuousLaw> {
        &self.laws
    }

    /// Stable directed dependencies from each state derivative to symbols it reads.
    pub fn dependency_graph(&self) -> BTreeMap<Identifier, BTreeSet<Identifier>> {
        self.laws
            .iter()
            .map(|(target, law)| (target.clone(), crate::expression_symbols(&law.expression)))
            .collect()
    }

    pub fn state_ids(&self) -> impl Iterator<Item = &Identifier> {
        self.variables
            .values()
            .filter(|variable| variable.role == VariableRole::State)
            .map(|variable| &variable.id)
    }
}

#[cfg(test)]
mod tests {
    use lawsynth_expr::Expr;

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn a_world_requires_laws_for_every_state() {
        let result = World::new([Variable::new(id("x"), VariableRole::State)], [], []);
        assert_eq!(result, Err(WorldError::StateVariableWithoutLaw(id("x"))));
    }

    #[test]
    fn a_minimal_world_is_well_formed() {
        let world = World::new(
            [Variable::new(id("x"), VariableRole::State)],
            [],
            [ContinuousLaw::new(id("x"), Expr::constant(-1.0))],
        )
        .unwrap();
        assert_eq!(world.state_ids().count(), 1);
    }

    #[test]
    fn symbols_have_a_single_namespace() {
        let result = World::new(
            [Variable::new(id("x"), VariableRole::State)],
            [Parameter::new(id("x"), 1.0)],
            [ContinuousLaw::new(id("x"), Expr::constant(-1.0))],
        );
        assert_eq!(result, Err(WorldError::ParameterConflictsWithVariable(id("x"))));
    }

    #[test]
    fn exposes_law_dependencies() {
        let world = World::new(
            [
                Variable::new(id("x"), VariableRole::State),
                Variable::new(id("y"), VariableRole::Control),
            ],
            [Parameter::new(id("rate"), 1.0)],
            [ContinuousLaw::new(
                id("x"),
                Expr::product(Expr::symbol(id("rate")), Expr::symbol(id("y"))),
            )],
        )
        .unwrap();
        assert_eq!(
            world.dependency_graph()[&id("x")].iter().cloned().collect::<Vec<_>>(),
            vec![id("rate"), id("y")]
        );
    }

    #[test]
    fn rejects_undeclared_law_symbols_by_default() {
        let result = World::new(
            [Variable::new(id("x"), VariableRole::State)],
            [],
            [ContinuousLaw::new(id("x"), Expr::symbol(id("missing")))],
        );
        assert_eq!(result, Err(WorldError::UnknownSymbol(id("missing"))));
    }

    #[test]
    fn can_explicitly_defer_symbol_validation_for_partial_worlds() {
        let world = World::new_with_config(
            [Variable::new(id("x"), VariableRole::State)],
            [],
            [ContinuousLaw::new(id("x"), Expr::symbol(id("external")))],
            WorldConfig { validate_expression_symbols: false, ..Default::default() },
        )
        .unwrap();
        assert_eq!(world.laws()[&id("x")].expression, Expr::symbol(id("external")));
    }
}
