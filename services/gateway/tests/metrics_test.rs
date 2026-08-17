mod support;

use lawsynth_gateway::HttpRequest;

#[test]
fn counters_track_totals_and_status_breakdown() {
    let gateway = support::gateway("127.0.0.1:9", 0);

    // Two local health checks (200) and one unknown route (404).
    gateway.handle(&HttpRequest::new("GET", "/healthz", Vec::new(), Vec::new()), "127.0.0.1");
    gateway.handle(&HttpRequest::new("GET", "/healthz", Vec::new(), Vec::new()), "127.0.0.1");
    gateway.handle(&HttpRequest::new("GET", "/v1/nope", Vec::new(), Vec::new()), "127.0.0.1");

    let snapshot = gateway.metrics_snapshot();
    assert_eq!(snapshot.by_status.get(&200), Some(&2));
    assert_eq!(snapshot.by_status.get(&404), Some(&1));
    assert!(snapshot.total >= 3);
}

#[test]
fn rate_limited_requests_are_counted() {
    let mut config = lawsynth_gateway::GatewayConfig::new("127.0.0.1:0", "127.0.0.1:9");
    config.rate_limit_quota = 1;
    let gateway = support::gateway_with(config, 0);

    let request = || HttpRequest::new("GET", "/v1/health", Vec::new(), Vec::new());
    gateway.handle(&request(), "10.0.0.1");
    gateway.handle(&request(), "10.0.0.1");

    assert_eq!(gateway.metrics_snapshot().rate_limited, 1);
}

#[test]
fn metrics_endpoint_renders_plain_text() {
    let gateway = support::gateway("127.0.0.1:9", 0);
    gateway.handle(&HttpRequest::new("GET", "/healthz", Vec::new(), Vec::new()), "127.0.0.1");

    let response =
        gateway.handle(&HttpRequest::new("GET", "/metrics", Vec::new(), Vec::new()), "127.0.0.1");
    assert_eq!(response.status, 200);
    let body = String::from_utf8(response.body).unwrap();
    assert!(body.contains("gateway_requests_total"));
    assert!(body.contains("gateway_responses_total{status=\"200\"}"));
}
