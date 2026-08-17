mod support;

use lawsynth_gateway::HttpRequest;

#[test]
fn healthz_is_served_locally_and_reports_ready() {
    let gateway = support::gateway("127.0.0.1:9", 0);
    let request = HttpRequest::new("GET", "/healthz", Vec::new(), Vec::new());
    let response = gateway.handle(&request, "127.0.0.1");
    assert_eq!(response.status, 200);
    let body = String::from_utf8(response.body).unwrap();
    assert!(body.contains("\"status\":\"ok\""));
    assert!(body.contains("\"service\":\"lawsynth-gateway\""));
}

#[test]
fn healthz_does_not_require_a_token_or_a_running_upstream() {
    // The configured upstream (127.0.0.1:9) is not listening, yet /healthz still
    // succeeds because it is answered by the gateway itself.
    let gateway = support::gateway("127.0.0.1:9", 0);
    let request = HttpRequest::new("GET", "/healthz", Vec::new(), Vec::new());
    assert_eq!(gateway.handle(&request, "127.0.0.1").status, 200);
}

#[test]
fn healthz_over_a_real_socket() {
    let gateway = support::gateway("127.0.0.1:9", 0);
    let (address, shutdown) = support::serve(gateway);
    let raw = support::round_trip(&address, b"GET /healthz HTTP/1.1\r\nHost: local\r\n\r\n");
    shutdown.trigger();
    assert!(raw.starts_with("HTTP/1.1 200 OK\r\n"), "unexpected: {raw}");
    assert!(raw.contains("\"status\":\"ok\""));
}
