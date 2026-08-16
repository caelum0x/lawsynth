use lawsynth_api_types::{ProjectId, SimulationRequest, TimeRange, WorldId, WorldRevision};
use std::hint::black_box;

fn main() {
    let world = WorldRevision::new(
        ProjectId::parse("bench").unwrap(),
        WorldId::parse("world").unwrap(),
        1,
        "a".repeat(64),
    )
    .unwrap();
    let time = TimeRange::new(0.0, 10.0, 0.01).unwrap();
    for seed in 0..100_000 {
        black_box(
            SimulationRequest::new(world.clone(), time, seed, vec!["x".into(), "v".into()])
                .unwrap(),
        );
    }
}
