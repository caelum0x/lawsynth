use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub struct Heartbeat {
    last_seen: Instant,
    sequence: u64,
}

impl Heartbeat {
    pub fn now() -> Self {
        Self { last_seen: Instant::now(), sequence: 0 }
    }
    pub fn beat(&mut self) -> u64 {
        self.sequence += 1;
        self.last_seen = Instant::now();
        self.sequence
    }
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }
    pub fn is_stale(&self, max_age: Duration) -> bool {
        self.last_seen.elapsed() > max_age
    }
}
