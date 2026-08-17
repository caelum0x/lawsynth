//! Cross-service integration: the gateway in front of a REAL artifact service.
//!
//! Unlike `proxy_test.rs` (which points the gateway at a one-shot canned mock),
//! these tests start an actual `lawsynth-artifact-service` HTTP server on an
//! ephemeral port and stand the gateway in front of it, then drive traffic over
//! raw client `TcpStream`s to the GATEWAY address. This proves the service layer
//! composes end to end: gateway admission (health, routing, auth, rate limiting)
//! plus real reverse-proxy forwarding into the artifact core over real sockets.
//!
//! PATH MAPPING. The gateway forwards the client's origin-form request target
//! *unchanged* to the upstream; it does not rewrite the path. The artifact
//! service mounts its routes at the root (`/artifacts`, `/artifacts/{id}`,
//! `/health`), while the gateway's route allowlist strips a configured
//! `api_prefix` before matching. Setting `api_prefix = "//"` makes the gateway's
//! `trim_end_matches('/')` collapse to an empty prefix, so the allowlist accepts
//! the artifact's *unprefixed* paths and forwards them verbatim. The gateway
//! therefore admits `POST /artifacts` and relays it straight onto the artifact's
//! `/artifacts` route. This is a real, validated configuration (`"//"` passes
//! `GatewayConfig::validate`), not a stub.

mod support;

use lawsynth_artifact_service::{ArtifactConfig, ArtifactServer, Clock as ArtifactClock, sha256};
use lawsynth_gateway::GatewayConfig;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

/// A pinned artifact clock so retention/expiry are fully controlled by the test.
const ARTIFACT_NOW: u64 = 1_000;

/// Owns a unique temp directory backing a real artifact service; cleans up on drop.
struct ArtifactRoot {
    path: PathBuf,
}

static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);

impl ArtifactRoot {
    fn new(label: &str) -> Self {
        let number = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir()
            .join(format!("lawsynth-gateway-artifact-{label}-{}-{number}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        Self { path }
    }
}

impl Drop for ArtifactRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Starts a real artifact HTTP server on an ephemeral port and returns its
/// address. The listener thread is detached; the returned `ArtifactRoot` must be
/// kept alive for the duration of the test to keep the backing store on disk.
fn start_artifact(label: &str) -> (String, ArtifactRoot) {
    let root = ArtifactRoot::new(label);
    let service = lawsynth_artifact_service::ArtifactService::open(ArtifactConfig::new(&root.path))
        .expect("artifact service opens");
    let clock: ArtifactClock = Arc::new(|| ARTIFACT_NOW);
    let server = ArtifactServer::new(service, clock);

    let listener = TcpListener::bind("127.0.0.1:0").expect("artifact binds ephemeral port");
    let address = listener.local_addr().unwrap().to_string();
    thread::spawn(move || {
        let _ = server.serve(&listener);
    });
    (address, root)
}

/// A minimally-parsed HTTP response: status code, raw header block, and body.
struct Response {
    status: u16,
    headers: String,
    body: Vec<u8>,
}

impl Response {
    fn header_contains(&self, needle: &str) -> bool {
        self.headers.to_ascii_lowercase().contains(&needle.to_ascii_lowercase())
    }

    fn body_text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

/// Connects to `address` with a small bounded retry loop (no wall-clock waits are
/// asserted on), sends `request`, and reads the full response to EOF. The gateway
/// and artifact both answer `Connection: close`, so read-to-end is complete.
fn send(address: &str, request: &[u8]) -> Response {
    let mut stream = connect_with_retry(address);
    stream.write_all(request).expect("write request");
    stream.flush().expect("flush request");
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).expect("read response");
    parse_response(&raw)
}

/// Opens a connection, retrying a bounded number of times. The listener is bound
/// before its serve thread is spawned, so a connection is accepted from the OS
/// backlog even in the brief window before `accept()` runs; the retry only guards
/// against scheduling jitter and never sleeps for correctness.
fn connect_with_retry(address: &str) -> TcpStream {
    let mut last_error = None;
    for _ in 0..50 {
        match TcpStream::connect(address) {
            Ok(stream) => return stream,
            Err(error) => {
                last_error = Some(error);
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
    panic!("could not connect to {address}: {last_error:?}");
}

/// Splits a raw HTTP/1.1 response into status code, header text, and body bytes.
fn parse_response(raw: &[u8]) -> Response {
    let split = raw.windows(4).position(|window| window == b"\r\n\r\n").expect("header terminator");
    let head = String::from_utf8_lossy(&raw[..split]).into_owned();
    let body = raw[split + 4..].to_vec();
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .expect("status code");
    let headers = head.split_once("\r\n").map(|(_, rest)| rest).unwrap_or("").to_owned();
    Response { status, headers, body }
}

/// Extracts a flat JSON string field value without pulling in a parser.
fn extract_string(body: &str, field: &str) -> String {
    let needle = format!("\"{field}\":\"");
    let start = body.find(&needle).expect("field present") + needle.len();
    let end = start + body[start..].find('"').expect("closing quote");
    body[start..end].to_owned()
}

/// Builds a gateway config that fronts `upstream` and admits the artifact's
/// unprefixed routes (see the module-level PATH MAPPING note).
fn gateway_config(upstream: &str) -> GatewayConfig {
    let mut config = GatewayConfig::new("127.0.0.1:0", upstream);
    config.api_prefix = "//".to_owned();
    config
}

#[test]
fn healthz_is_answered_locally_and_not_proxied() {
    // Even with a real artifact upstream running, /healthz is the gateway's own
    // liveness endpoint and must never be forwarded.
    let (artifact, _root) = start_artifact("healthz");
    let gateway = support::gateway_with(gateway_config(&artifact), 0);
    let (address, shutdown) = support::serve(gateway);

    let response = send(&address, b"GET /healthz HTTP/1.1\r\nHost: edge\r\n\r\n");
    shutdown.trigger();

    assert_eq!(response.status, 200);
    // The gateway stamps a correlation id on responses it produces itself.
    assert!(response.header_contains("x-request-id"), "missing request id: {}", response.headers);
    let body = response.body_text();
    assert!(body.contains("\"service\":\"lawsynth-gateway\""), "not the gateway health: {body}");
    // The artifact's own health payload (artifact_count/capacity) must be absent.
    assert!(!body.contains("artifact_count"), "healthz leaked upstream health: {body}");
}

#[test]
fn artifact_create_and_fetch_flow_through_the_gateway() {
    // (b) POST flows gateway -> artifact and returns 201 with the content id.
    // (c) GET fetches the same bytes back THROUGH the gateway.
    let (artifact, _root) = start_artifact("create-fetch");
    let gateway = support::gateway_with(gateway_config(&artifact), 0);
    let (address, shutdown) = support::serve(gateway);

    let payload = b"hello cross-service integration";
    let create_request = format!(
        "POST /artifacts HTTP/1.1\r\nHost: edge\r\nAuthorization: Bearer test-token\r\n\
         Content-Type: text/plain\r\nContent-Length: {}\r\n\r\n",
        payload.len()
    );
    let mut create_bytes = create_request.into_bytes();
    create_bytes.extend_from_slice(payload);

    let created = send(&address, &create_bytes);
    assert_eq!(created.status, 201, "create body: {}", created.body_text());

    let expected_id = sha256(payload);
    let created_body = created.body_text();
    let returned_id = extract_string(&created_body, "id");
    assert_eq!(returned_id, expected_id, "content-addressed id mismatch: {created_body}");
    // The artifact's Location header is relayed through the gateway unchanged.
    assert!(
        created.header_contains(&format!("location: /artifacts/{expected_id}")),
        "missing/incorrect Location: {}",
        created.headers
    );

    // (c) Fetch the artifact back THROUGH the gateway; the bytes round-trip.
    let fetch_request = format!(
        "GET /artifacts/{expected_id} HTTP/1.1\r\nHost: edge\r\nAuthorization: Bearer test-token\r\n\r\n"
    );
    let fetched = send(&address, fetch_request.as_bytes());
    shutdown.trigger();

    assert_eq!(fetched.status, 200, "fetch headers: {}", fetched.headers);
    assert_eq!(fetched.body, payload, "artifact bytes not relayed through the gateway");
    assert!(fetched.header_contains("content-type: text/plain"), "headers: {}", fetched.headers);
}

#[test]
fn unknown_route_is_404_at_the_gateway_before_the_upstream() {
    // (d) An unmatched path is rejected at the edge with the gateway's own 404
    // envelope and correlation id, so it never reaches the artifact upstream.
    let (artifact, _root) = start_artifact("unknown");
    let gateway = support::gateway_with(gateway_config(&artifact), 0);
    let (address, shutdown) = support::serve(gateway);

    let response = send(&address, b"GET /not-a-real-route HTTP/1.1\r\nHost: edge\r\n\r\n");
    shutdown.trigger();

    assert_eq!(response.status, 404);
    assert!(
        response.header_contains("x-request-id"),
        "gateway did not answer: {}",
        response.headers
    );
    let body = response.body_text();
    // The gateway's 404 message is distinct from the artifact's ("...request"
    // vs. "...request path"), proving this response is edge-originated.
    assert!(
        body.contains("\"message\":\"no route matches the request\""),
        "not the gateway 404: {body}"
    );
}

#[test]
fn rate_limit_returns_429_after_the_quota_is_exhausted() {
    // (e) With a fixed clock the rate-limit window never advances, so the
    // (quota + 1)-th request from the same client IP is rejected with 429.
    let (artifact, _root) = start_artifact("rate-limit");
    let mut config = gateway_config(&artifact);
    config.rate_limit_quota = 2;
    let gateway = support::gateway_with(config, 0);
    let (address, shutdown) = support::serve(gateway);

    // The public /health route flows through the limiter (unlike /healthz).
    let health_request = b"GET /health HTTP/1.1\r\nHost: edge\r\n\r\n";
    let first = send(&address, health_request);
    let second = send(&address, health_request);
    let third = send(&address, health_request);
    shutdown.trigger();

    assert_eq!(first.status, 200, "first within quota: {}", first.body_text());
    assert_eq!(second.status, 200, "second within quota: {}", second.body_text());
    assert_eq!(third.status, 429, "third should exceed quota: {}", third.body_text());
    assert!(third.header_contains("retry-after"), "429 missing Retry-After: {}", third.headers);
    assert!(third.body_text().contains("\"code\":\"rate_limited\""));
}
