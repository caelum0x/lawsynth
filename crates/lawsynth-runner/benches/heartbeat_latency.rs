use lawsynth_runner::Heartbeat;
use std::hint::black_box;

fn main() {
    let mut heartbeat = Heartbeat::now();
    for _ in 0..100_000 {
        black_box(heartbeat.beat());
    }
}
