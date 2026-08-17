mod support;

use lawsynth_gateway::proxy::forward;
use lawsynth_gateway::{GatewayConfig, HttpRequest};

#[test]
fn end_to_end_proxy_over_real_sockets() {
    // A mock upstream returns a canned body; the gateway forwards a client
    // request to it over a real socket and relays the response back.
    let (upstream, upstream_handle) = support::spawn_upstream(
        b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 11\r\n\r\nhello-world"
            .to_vec(),
    );
    let gateway = support::gateway(&upstream, 0);
    let (address, shutdown) = support::serve(gateway);

    let raw = support::round_trip(&address, b"GET /v1/health HTTP/1.1\r\nHost: edge\r\n\r\n");
    shutdown.trigger();

    assert!(raw.starts_with("HTTP/1.1 200 OK\r\n"), "unexpected response: {raw}");
    assert!(raw.ends_with("\r\n\r\nhello-world"), "body not relayed: {raw}");

    let received = String::from_utf8(upstream_handle.join().unwrap()).unwrap();
    assert!(received.starts_with("GET /v1/health HTTP/1.1\r\n"));
    assert!(received.contains("X-Forwarded-For: 127.0.0.1"));
    assert!(received.contains("X-Forwarded-Proto: https"));
}

#[test]
fn large_body_is_streamed_through_the_proxy_in_both_directions() {
    // A 512 KiB upload is forwarded to the upstream, and a 512 KiB download is
    // relayed back, exercising the bounded chunked copy on both legs.
    let download = vec![b'D'; 512 * 1024];
    let mut canned = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
        download.len()
    )
    .into_bytes();
    canned.extend_from_slice(&download);
    let (upstream, upstream_handle) = support::spawn_upstream(canned);

    let config = GatewayConfig::new("127.0.0.1:0", upstream);
    let upload = vec![b'U'; 512 * 1024];
    let request = HttpRequest::new(
        "POST",
        "/v1/artifacts",
        vec![("Content-Type".into(), "application/octet-stream".into())],
        upload.clone(),
    );

    let response = forward(&config, &request, "203.0.113.7").unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(response.body.len(), download.len());
    assert_eq!(response.body, download);

    let received = upstream_handle.join().unwrap();
    // The upstream received the full upload body after the request head.
    assert!(received.windows(upload.len()).any(|window| window == upload.as_slice()));
}

#[test]
fn hop_by_hop_headers_are_not_relayed_upstream() {
    let (upstream, upstream_handle) =
        support::spawn_upstream(b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n".to_vec());
    let config = GatewayConfig::new("127.0.0.1:0", upstream);
    let request = HttpRequest::new(
        "GET",
        "/v1/health",
        vec![("Connection".into(), "keep-alive".into()), ("Upgrade".into(), "websocket".into())],
        Vec::new(),
    );

    let response = forward(&config, &request, "1.2.3.4").unwrap();
    assert_eq!(response.status, 204);

    let received = String::from_utf8(upstream_handle.join().unwrap()).unwrap().to_ascii_lowercase();
    assert!(!received.contains("upgrade: websocket"));
    assert!(!received.contains("connection: keep-alive"));
}

#[test]
fn unreachable_upstream_yields_bad_gateway() {
    let config = GatewayConfig::new("127.0.0.1:0", "127.0.0.1:1");
    let request = HttpRequest::new("POST", "/v1/runs", Vec::new(), Vec::new());
    let error = forward(&config, &request, "1.1.1.1").unwrap_err();
    assert_eq!(error.status(), 502);
}
