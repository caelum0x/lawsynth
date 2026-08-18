use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

#[test]
fn world_exposes_variables_and_law_dependencies_in_stable_identifier_order() {
    let x = Identifier::new("x").unwrap();
    let control = Identifier::new("u").unwrap();
    let world = World::new(
        [
            Variable::new(x.clone(), VariableRole::State),
            Variable::new(control.clone(), VariableRole::Control),
        ],
        [],
        [ContinuousLaw::new(
            x.clone(),
            Expr::sum(Expr::symbol(x.clone()), Expr::symbol(control.clone())),
        )],
    )
    .unwrap();
    assert_eq!(world.state_ids().cloned().collect::<Vec<_>>(), vec![x.clone()]);
    assert_eq!(world.dependency_graph()[&x].iter().cloned().collect::<Vec<_>>(), vec![control, x]);
}
