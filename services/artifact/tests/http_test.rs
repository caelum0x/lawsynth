//! Transport-level tests for the artifact HTTP server.
//!
//! Most cases drive the router through `ArtifactServer::handle` with a fixed
//! clock, keeping them deterministic and socket-free. One case exercises the
//! real `std::net` accept loop end to end.

mod support;

use lawsynth_artifact_service::{ArtifactServer, Clock, HttpRequest, HttpResponse, sha256};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

/// Builds a server whose clock is pinned to `now`, so retention and expiry are
/// fully controlled by the test.
fn server_at(root: &support::TestRoot, now: u64) -> ArtifactServer {
    let clock: Clock = Arc::new(move || now);
    ArtifactServer::new(root.service(), clock)
}

fn get(path: &str) -> HttpRequest {
    HttpRequest::new("GET", path, Vec::new(), Vec::new())
}

fn post(path: &str, headers: Vec<(String, String)>, body: &[u8]) -> HttpRequest {
    HttpRequest::new("POST", path, headers, body.to_vec())
}

fn body_text(response: &HttpResponse) -> String {
    String::from_utf8(response.body.clone()).unwrap()
}

#[test]
fn full_lifecycle_over_the_router() {
    let root = support::TestRoot::new("http-lifecycle");
    let server = server_at(&root, 1_000);

    let health = server.handle(&get("/health"));
    assert_eq!(health.status, 200);
    assert!(body_text(&health).contains("\"artifact_count\":0"));

    let created = server.handle(&post(
        "/artifacts",
        vec![("Content-Type".into(), "text/plain".into())],
        b"hello",
    ));
    assert_eq!(created.status, 201);
    let id = sha256(b"hello");
    assert_eq!(
        created
            .headers
            .iter()
            .find(|(name, _)| name == "Location")
            .map(|(_, value)| value.as_str()),
        Some(format!("/artifacts/{id}").as_str())
    );
    assert!(body_text(&created).contains(&format!("\"id\":\"{id}\"")));

    let fetched = server.handle(&get(&format!("/artifacts/{id}")));
    assert_eq!(fetched.status, 200);
    assert_eq!(fetched.body, b"hello");
    assert_eq!(
        fetched.headers.iter().find(|(name, _)| name == "Content-Type").map(|(_, v)| v.as_str()),
        Some("text/plain")
    );

    let metadata = server.handle(&get(&format!("/artifacts/{id}/metadata")));
    assert_eq!(metadata.status, 200);
    assert!(body_text(&metadata).contains("\"size_bytes\":5"));
    assert!(body_text(&metadata).contains("\"content_type\":\"text/plain\""));

    let deleted = HttpRequest::new("DELETE", &format!("/artifacts/{id}"), Vec::new(), Vec::new());
    assert_eq!(server.handle(&deleted).status, 204);
    assert_eq!(server.handle(&get(&format!("/artifacts/{id}"))).status, 404);
}

#[test]
fn ingest_is_content_addressed_and_idempotent() {
    let root = support::TestRoot::new("http-idempotent");
    let server = server_at(&root, 1);
    let first = server.handle(&post("/artifacts", Vec::new(), b"same"));
    let second = server.handle(&post("/artifacts", Vec::new(), b"same"));
    assert_eq!(first.status, 201);
    assert_eq!(second.status, 201);
    assert_eq!(first.body, second.body);
}

#[test]
fn multipart_upload_assembles_and_publishes() {
    let root = support::TestRoot::new("http-multipart");
    let server = server_at(&root, 5);

    let begin = server.handle(&post("/uploads", Vec::new(), b""));
    assert_eq!(begin.status, 201);
    let upload_id = extract_string(&body_text(&begin), "upload_id");

    let part = |number: u32, bytes: &[u8]| {
        HttpRequest::new(
            "PUT",
            &format!("/uploads/{upload_id}/parts/{number}"),
            Vec::new(),
            bytes.to_vec(),
        )
    };
    assert_eq!(server.handle(&part(1, b"foo")).status, 204);
    assert_eq!(server.handle(&part(2, b"bar")).status, 204);

    let complete = server.handle(&post(&format!("/uploads/{upload_id}/complete"), Vec::new(), b""));
    assert_eq!(complete.status, 201);
    let id = sha256(b"foobar");
    assert_eq!(server.handle(&get(&format!("/artifacts/{id}"))).body, b"foobar");
}

#[test]
fn errors_map_to_documented_statuses() {
    let root = support::TestRoot::new("http-errors");
    let server = server_at(&root, 100);

    // Not a valid SHA-256 address -> 400.
    let invalid = server.handle(&get("/artifacts/not-a-hash"));
    assert_eq!(invalid.status, 400);
    assert!(body_text(&invalid).contains("\"code\":\"invalid_artifact_id\""));

    // Well-formed but absent -> 404.
    let missing = server.handle(&get(&format!("/artifacts/{}", sha256(b"absent"))));
    assert_eq!(missing.status, 404);

    // Wrong method on a known route -> 405 with Allow.
    let bad_method = server.handle(&HttpRequest::new("PUT", "/health", Vec::new(), Vec::new()));
    assert_eq!(bad_method.status, 405);
    assert!(bad_method.headers.iter().any(|(name, _)| name == "Allow"));

    // Unknown route -> 404.
    assert_eq!(server.handle(&get("/nope")).status, 404);

    // Malformed retention header -> 400.
    let bad_retention = server.handle(&post(
        "/artifacts",
        vec![("X-Retention-Expires-At".into(), "soon".into())],
        b"x",
    ));
    assert_eq!(bad_retention.status, 400);
}

#[test]
fn expired_artifacts_report_gone_and_are_collectable() {
    let root = support::TestRoot::new("http-expiry");
    let server = server_at(&root, 10);
    let created = server.handle(&post(
        "/artifacts",
        vec![("X-Retention-Expires-At".into(), "20".into())],
        b"ephemeral",
    ));
    assert_eq!(created.status, 201);
    let id = sha256(b"ephemeral");

    // A clock past expiry reports the artifact as gone.
    let later = server_at(&root, 999);
    assert_eq!(later.handle(&get(&format!("/artifacts/{id}"))).status, 410);

    let gc = later.handle(&post("/gc", Vec::new(), b""));
    assert_eq!(gc.status, 200);
    assert!(body_text(&gc).contains(&id));
}

#[test]
fn serves_a_request_over_a_real_socket() {
    let root = support::TestRoot::new("http-socket");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let clock: Clock = Arc::new(|| 1);
    let server = ArtifactServer::new(root.service(), clock);
    thread::spawn(move || {
        let _ = server.serve(&listener);
    });

    let raw = read_over_socket(
        address.to_string().as_str(),
        b"POST /artifacts HTTP/1.1\r\nHost: local\r\nContent-Length: 5\r\n\r\nhello",
    );
    assert!(raw.starts_with("HTTP/1.1 201 Created\r\n"), "unexpected response: {raw}");

    let id = sha256(b"hello");
    let fetched = read_over_socket(
        address.to_string().as_str(),
        format!("GET /artifacts/{id} HTTP/1.1\r\nHost: local\r\n\r\n").as_bytes(),
    );
    assert!(fetched.starts_with("HTTP/1.1 200 OK\r\n"), "unexpected response: {fetched}");
    assert!(fetched.ends_with("\r\n\r\nhello"), "body not delivered: {fetched}");
}

fn read_over_socket(address: &str, request: &[u8]) -> String {
    let mut stream = TcpStream::connect(address).unwrap();
    stream.write_all(request).unwrap();
    stream.flush().unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

/// Extracts a JSON string field value from a flat object body without a parser.
fn extract_string(body: &str, field: &str) -> String {
    let needle = format!("\"{field}\":\"");
    let start = body.find(&needle).expect("field present") + needle.len();
    let end = start + body[start..].find('"').expect("closing quote");
    body[start..end].to_owned()
}
