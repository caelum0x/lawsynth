use lawsynth_runner::{ResourceLimiter, ResourceRequest};

#[test]
fn resource_reservations_cannot_overcommit_or_underflow() {
    let capacity = ResourceRequest::new(1000, 4096, 100).unwrap();
    let request = ResourceRequest::new(500, 2048, 10).unwrap();
    let mut limiter = ResourceLimiter::new(capacity);
    limiter.reserve(request).unwrap();
    assert_eq!(limiter.available().memory_bytes, 2048);
    assert!(limiter.reserve(request).is_ok());
    assert!(
        limiter
            .reserve(ResourceRequest::new(1, 1, 1).unwrap())
            .is_err()
    );
    limiter.release(request).unwrap();
    limiter.release(request).unwrap();
    assert!(limiter.release(request).is_err());
}
