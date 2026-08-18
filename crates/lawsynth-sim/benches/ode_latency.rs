use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_sim::{SimulationConfig, SimulationRequest, simulate};
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};
use std::{hint::black_box, time::Instant};

fn world() -> (World, Identifier) {
    let x = Identifier::new("x").unwrap();
    let world = World::new(
        [Variable::new(x.clone(), VariableRole::State)],
        [],
        [ContinuousLaw::new(
            x.clone(),
            Expr::unary(lawsynth_expr::UnaryOperator::Negate, Expr::symbol(x.clone())),
        )],
    )
    .unwrap();
    (world, x)
}
fn main() {
    let (world, x) = world();
    let config = SimulationConfig::new(0.0, 2.0, 0.01).unwrap();
    let request = SimulationRequest::default().with_initial(x, 1.0);
    let started = Instant::now();
    let mut samples = 0;
    for _ in 0..100 {
        samples += black_box(simulate(&world, config, &request).unwrap()).samples();
    }
    println!("integrated {samples} RK4 samples in {:?}", started.elapsed());
}
