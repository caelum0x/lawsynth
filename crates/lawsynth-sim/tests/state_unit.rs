use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_sim::{SimulationConfig, SimulationRequest, simulate};
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

#[test]
fn simulator_keeps_state_vectors_aligned_in_trajectory_storage() {
    let x = Identifier::new("x").unwrap();
    let world = World::new(
        [Variable::new(x.clone(), VariableRole::State)],
        [],
        [ContinuousLaw::new(x.clone(), Expr::constant(1.0))],
    )
    .unwrap();
    let trajectory = simulate(
        &world,
        SimulationConfig::new(0.0, 1.0, 0.25).unwrap(),
        &SimulationRequest::default().with_initial(x.clone(), 0.0),
    )
    .unwrap();
    assert_eq!(trajectory.values[&x].len(), trajectory.samples());
    assert_eq!(trajectory.values[&x], vec![0.0, 0.25, 0.5, 0.75, 1.0]);
}
