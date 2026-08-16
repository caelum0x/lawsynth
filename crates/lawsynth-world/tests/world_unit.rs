use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World, WorldConfig};

#[test]
fn world_configuration_controls_construction_time_symbol_validation() {
    let x = Identifier::new("x").unwrap();
    let deferred = World::new_with_config(
        [Variable::new(x.clone(), VariableRole::State)],
        [],
        [ContinuousLaw::new(
            x,
            Expr::symbol(Identifier::new("future_input").unwrap()),
        )],
        WorldConfig {
            validate_expression_symbols: false,
            validate_units: true,
        },
    );
    assert!(deferred.is_ok());
}
