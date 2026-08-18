use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_sim::{CompiledContinuousWorld, CompiledDiscreteWorld};
use lawsynth_world::{ContinuousLaw, DiscreteLaw, DiscreteWorld, Variable, VariableRole, World};

#[test]
fn compiled_plans_preserve_world_law_order_and_targets() {
    let x = Identifier::new("x").unwrap();
    let continuous = World::new(
        [Variable::new(x.clone(), VariableRole::State)],
        [],
        [ContinuousLaw::new(x.clone(), Expr::constant(1.0))],
    )
    .unwrap();
    let discrete = DiscreteWorld::new(
        [Variable::new(x.clone(), VariableRole::State)],
        [],
        [DiscreteLaw::new(x.clone(), Expr::constant(1.0))],
    )
    .unwrap();
    assert_eq!(
        CompiledContinuousWorld::compile(&continuous).law_targets().collect::<Vec<_>>(),
        vec![&x]
    );
    assert_eq!(
        CompiledDiscreteWorld::compile(&discrete).law_targets().collect::<Vec<_>>(),
        vec![&x]
    );
}
