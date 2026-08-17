use lawsynth_gateway::tracing::{RequestIds, request_log_line};

#[test]
fn request_ids_are_deterministic_and_monotonic() {
    let ids = RequestIds::new();
    assert_eq!(ids.next_id(), "req-0000000000000001");
    assert_eq!(ids.next_id(), "req-0000000000000002");
    assert_eq!(ids.next_id(), "req-0000000000000003");
}

#[test]
fn request_ids_are_unique_across_the_generator() {
    let ids = RequestIds::new();
    let first = ids.next_id();
    let second = ids.next_id();
    assert_ne!(first, second);
}

#[test]
fn log_line_is_structured_and_complete() {
    let line = request_log_line("req-42", "POST", "/v1/runs", 201, "10.0.0.5", 2048);
    for fragment in [
        "request_id=req-42",
        "method=POST",
        "path=/v1/runs",
        "status=201",
        "client=10.0.0.5",
        "duration_us=2048",
    ] {
        assert!(line.contains(fragment), "missing {fragment} in {line}");
    }
}
