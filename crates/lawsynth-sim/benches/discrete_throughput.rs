use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_sim::{DiscreteSimulationConfig, SimulationRequest, simulate_discrete};
use lawsynth_world::{DiscreteLaw, DiscreteWorld, Variable, VariableRole};
use std::{hint::black_box, time::Instant};

fn main() {
    let x = Identifier::new("x").unwrap();
    let world = DiscreteWorld::new(
        [Variable::new(x.clone(), VariableRole::State)],
        [],
        [DiscreteLaw::new(x.clone(), Expr::sum(Expr::symbol(x.clone()), Expr::constant(1.0)))],
    )
    .unwrap();
    let config = DiscreteSimulationConfig::new(0.0, 1_000).unwrap();
    let request = SimulationRequest::default().with_initial(x, 0.0);
    let started = Instant::now();
    let mut samples = 0;
    for _ in 0..100 {
        samples += black_box(simulate_discrete(&world, config, &request).unwrap()).samples();
    }
    println!("simulated {samples} recurrence samples in {:?}", started.elapsed());
}
