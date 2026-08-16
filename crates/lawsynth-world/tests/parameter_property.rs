use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_world::{
    ContinuousLaw, Parameter, Variable, VariableRole, World, WorldConfig, WorldError,
};

#[test]
fn parameter_validation_is_finite_and_can_explicitly_defer_symbol_validation() {
    let x = Identifier::new("x").unwrap();
    let rate = Identifier::new("rate").unwrap();
    assert_eq!(
        World::new(
            [Variable::new(x.clone(), VariableRole::State)],
            [Parameter::new(rate.clone(), f64::INFINITY)],
            [ContinuousLaw::new(x.clone(), Expr::symbol(rate.clone()))],
        ),
        Err(WorldError::NonFiniteParameter(rate.clone()))
    );
    let deferred = World::new_with_config(
        [Variable::new(x.clone(), VariableRole::State)],
        [],
        [ContinuousLaw::new(x, Expr::symbol(rate))],
        WorldConfig {
            validate_expression_symbols: false,
            validate_units: false,
        },
    );
    assert!(deferred.is_ok());
}
