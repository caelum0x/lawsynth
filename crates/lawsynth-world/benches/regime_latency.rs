use lawsynth_core::Identifier;
use lawsynth_world::{RegimeInterval, RegimeSchedule};
use std::{hint::black_box, time::Instant};

fn main() {
    let schedule = RegimeSchedule::new(vec![
        RegimeInterval { regime: Identifier::new("warm").unwrap(), start: 0.0, end: 10.0 },
        RegimeInterval { regime: Identifier::new("cold").unwrap(), start: 10.0, end: 20.0 },
    ])
    .unwrap();
    let started = Instant::now();
    let mut hits = 0;
    for i in 0..1_000_000 {
        hits += usize::from(black_box(schedule.active_at((i % 2_000) as f64 / 100.0)).is_some());
    }
    println!("located {hits} regimes in {:?}", started.elapsed());
}
