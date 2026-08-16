use lawsynth_runner::{ResourceRequest, WorkEnvelope};

#[test]
fn envelopes_reject_expired_shape_and_preserve_deadline() {
    let resources = ResourceRequest::new(10, 64, 0).unwrap();
    let envelope = WorkEnvelope::new("a_1", "simulate", 1, 100, 101, resources, vec![]).unwrap();
    assert!(!envelope.is_expired(100));
    assert!(envelope.is_expired(101));
    assert!(WorkEnvelope::new("bad id", "simulate", 1, 1, 2, resources, vec![]).is_err());
}
