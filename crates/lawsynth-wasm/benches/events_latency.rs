use lawsynth_wasm::{Event, EventDirection, Expression};
use std::hint::black_box;
use std::time::Instant;
fn main() {
    let event = Event::new(
        "crossing",
        Expression::parse("x").unwrap(),
        EventDirection::Rising,
    )
    .unwrap();
    let start = Instant::now();
    let mut crossings = 0;
    for index in 0..1_000_000 {
        if event.crosses(index as f64 - 500_000.5, index as f64 - 499_999.5) {
            crossings += 1;
        }
    }
    black_box(crossings);
    println!(
        "event crossings: {:.0} checks/s",
        1_000_000.0 / start.elapsed().as_secs_f64()
    );
}
