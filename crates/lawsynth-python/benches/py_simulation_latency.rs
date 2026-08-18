#[path = "../src/py_events.rs"]
mod py_events;

use py_events::scheduled_values;
use std::{hint::black_box, time::Instant};
fn main() {
    let started = Instant::now();
    let mut changes = 0;
    for _ in 0..100_000 {
        changes += black_box(scheduled_values([(0.0, "gain".to_owned(), 1.0)]).unwrap()).len();
    }
    println!("converted {changes} scheduled boundary values in {:?}", started.elapsed());
}
