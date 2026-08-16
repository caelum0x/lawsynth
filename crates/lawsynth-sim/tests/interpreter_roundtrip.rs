use std::collections::BTreeMap;

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_sim::{CompiledContinuousWorld, SimulationContext, evaluate_continuous};
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

#[test]
fn compiled_interpreter_evaluates_the_same_law_as_the_world_definition() {
    let x = Identifier::new("x").unwrap();
    let world = World::new(
        [Variable::new(x.clone(), VariableRole::State)],
        [],
        [ContinuousLaw::new(
            x.clone(),
            Expr::product(Expr::constant(2.0), Expr::symbol(x.clone())),
        )],
    )
    .unwrap();
    let values = evaluate_continuous(
        &CompiledContinuousWorld::compile(&world),
        &SimulationContext::new(
            BTreeMap::from([(x.clone(), 1.5)]),
            BTreeMap::new(),
            BTreeMap::new(),
        ),
    )
    .unwrap();
    assert_eq!(values[&x], 3.0);
}
