use lawsynth_runner::{ResourceLimiter, ResourceRequest};
use std::hint::black_box;

fn main() {
    let request = ResourceRequest::new(1, 1, 0).unwrap();
    let mut limiter = ResourceLimiter::new(ResourceRequest::new(1_000_000, 1_000_000, 0).unwrap());
    for _ in 0..100_000 {
        limiter.reserve(request).unwrap();
        black_box(limiter.available());
        limiter.release(request).unwrap();
    }
}
