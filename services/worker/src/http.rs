//! A dependency-free HTTP/1.1 status transport for the worker service.
//!
//! The worker core deliberately links no async runtime or HTTP framework, so
//! this module implements a small, blocking, thread-per-connection server on
//! `std::net`. It parses one request per connection, routes it through
//! [`crate::router`], writes the response, and closes the connection.
//!
//! HONESTY BOUNDARY: this transport exposes only serializable observability and
//! control -- readiness, admission/config limits, and the durable lifecycle
//! checkpoints of jobs the worker has already run. It never accepts executable
//! [`crate::JobEnvelope`]s, because those carry typed, in-memory payloads with no
//! wire codec. See [`crate::TransportSurface::HttpStatus`] for the advertised
//! contract.
//!
//! Time is provided by an injected clock so the routing layer stays
//! deterministic and testable; [`WorkerServer::with_system_clock`] wires the
//! real wall clock for `main`.

use crate::{WorkerError, http_error, json::Json, router};
use lawsynth_store::ObjectStore;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

use crate::Worker;

/// Supplies the current Unix time in seconds to time-dependent operations.
pub type Clock = Arc<dyn Fn() -> u64 + Send + Sync>;

/// Absolute ceiling on a request line + header block, guarding against a client
/// that never sends the terminating blank line.
const MAX_HEADER_BYTES: usize = 64 * 1024;

/// The status surface exposes read-only endpoints that carry no request body;
/// any declared body beyond this small ceiling is rejected before it is read.
const MAX_REQUEST_BODY_BYTES: usize = 64 * 1024;

/// A parsed HTTP request. Header names are stored lowercased for case-insensitive
/// lookup; the body is bounded by the server's configured maximum.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    /// Constructs a request directly, primarily for routing tests.
    pub fn new(
        method: impl Into<String>,
        target: &str,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    ) -> Self {
        let (path, query) = split_target(target);
        Self {
            method: method.into(),
            path,
            query,
            headers: headers
                .into_iter()
                .map(|(name, value)| (name.to_ascii_lowercase(), value))
                .collect(),
            body,
        }
    }

    /// Returns the first value for a header, matched case-insensitively.
    pub fn header(&self, name: &str) -> Option<&str> {
        let name = name.to_ascii_lowercase();
        self.headers.iter().find(|(key, _)| *key == name).map(|(_, value)| value.as_str())
    }

    /// Returns the first value of a query-string parameter, if present.
    pub fn query_param(&self, key: &str) -> Option<String> {
        self.query.split('&').find_map(|pair| {
            let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
            (name == key).then(|| value.to_owned())
        })
    }
}

/// An HTTP response ready to be written to the wire.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn new(status: u16) -> Self {
        Self { status, headers: Vec::new(), body: Vec::new() }
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    pub fn empty(status: u16) -> Self {
        Self::new(status)
    }

    /// Body of raw bytes served with an explicit content type.
    pub fn bytes(status: u16, content_type: &str, body: Vec<u8>) -> Self {
        Self::new(status).with_header("Content-Type", content_type).body(body)
    }

    /// Renders a JSON value as an `application/json` body.
    pub fn json(status: u16, value: &Json) -> Self {
        Self::bytes(status, "application/json", value.render().into_bytes())
    }

    /// Maps a domain error to its documented status and machine-readable body.
    pub fn error(error: &WorkerError) -> Self {
        let classified = http_error::classify(error);
        Self::json(classified.status, &http_error::body(error))
    }

    /// Builds an error response from a transport-level code discovered before a
    /// domain call (malformed method, unmatched route, invalid job id).
    pub fn error_code(status: u16, code: &'static str, message: &str) -> Self {
        Self::json(
            status,
            &Json::Object(vec![
                ("code".into(), Json::string(code)),
                ("message".into(), Json::string(message)),
            ]),
        )
    }

    fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }
}

/// Outcome of reading a single request from a connection.
enum ReadOutcome {
    Request(HttpRequest),
    PayloadTooLarge,
    Closed,
}

/// A blocking HTTP status transport bound to a shared [`Worker`].
///
/// The worker is held behind an [`Arc`] so the server can be cheaply cloned for
/// each accepted connection while every clone observes the same durable
/// checkpoints and admission state.
pub struct WorkerServer<S> {
    worker: Arc<Worker<S>>,
    clock: Clock,
    max_body_bytes: usize,
}

// A manual `Clone` avoids requiring `S: Clone`; only the `Arc` is cloned.
impl<S> Clone for WorkerServer<S> {
    fn clone(&self) -> Self {
        Self {
            worker: Arc::clone(&self.worker),
            clock: Arc::clone(&self.clock),
            max_body_bytes: self.max_body_bytes,
        }
    }
}

impl<S: ObjectStore> WorkerServer<S> {
    /// Builds a server with an explicit clock; tests inject a fixed value here.
    pub fn new(worker: Arc<Worker<S>>, clock: Clock) -> Self {
        Self { worker, clock, max_body_bytes: MAX_REQUEST_BODY_BYTES }
    }

    /// Builds a server whose clock reads the system wall clock in Unix seconds.
    pub fn with_system_clock(worker: Arc<Worker<S>>) -> Self {
        let clock: Clock = Arc::new(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|elapsed| elapsed.as_secs())
                .unwrap_or(0)
        });
        Self::new(worker, clock)
    }

    /// Routes an already-parsed request, used by tests that bypass sockets.
    pub fn handle(&self, request: &HttpRequest) -> HttpResponse {
        router::route(&self.worker, (self.clock)(), request)
    }
}

impl<S: ObjectStore + 'static> WorkerServer<S> {
    /// Serves connections until the listener errors, one thread per connection.
    pub fn serve(&self, listener: &TcpListener) -> io::Result<()> {
        for stream in listener.incoming() {
            let stream = stream?;
            let server = self.clone();
            thread::spawn(move || {
                if let Err(error) = server.serve_connection(stream) {
                    eprintln!("lawsynth-worker: connection error: {error}");
                }
            });
        }
        Ok(())
    }

    fn serve_connection(&self, stream: TcpStream) -> io::Result<()> {
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut writer = stream;
        let response = match read_request(&mut reader, self.max_body_bytes)? {
            ReadOutcome::Closed => return Ok(()),
            ReadOutcome::PayloadTooLarge => HttpResponse::error_code(
                413,
                "payload_too_large",
                "request body exceeds the configured maximum",
            ),
            ReadOutcome::Request(request) => router::route(&self.worker, (self.clock)(), &request),
        };
        write_response(&mut writer, &response)?;
        writer.flush()
    }
}

/// Splits a request target into its path and raw query components.
fn split_target(target: &str) -> (String, String) {
    match target.split_once('?') {
        Some((path, query)) => (path.to_owned(), query.to_owned()),
        None => (target.to_owned(), String::new()),
    }
}

/// Reads and parses a single request, enforcing header and body size ceilings.
fn read_request<R: BufRead>(reader: &mut R, max_body: usize) -> io::Result<ReadOutcome> {
    let mut header_block = Vec::new();
    loop {
        let mut line = Vec::new();
        let read = reader.read_until(b'\n', &mut line)?;
        if read == 0 {
            // Clean EOF before any bytes: the peer closed the connection.
            return Ok(ReadOutcome::Closed);
        }
        if line == b"\r\n" || line == b"\n" {
            break;
        }
        header_block.extend_from_slice(&line);
        if header_block.len() > MAX_HEADER_BYTES {
            return Ok(ReadOutcome::PayloadTooLarge);
        }
    }

    let text = String::from_utf8_lossy(&header_block);
    let mut lines = text.lines();
    let Some(request_line) = lines.next() else {
        return Ok(ReadOutcome::Closed);
    };
    let mut parts = request_line.split_whitespace();
    let (Some(method), Some(target)) = (parts.next(), parts.next()) else {
        return Ok(ReadOutcome::Closed);
    };

    let mut headers = Vec::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_ascii_lowercase(), value.trim().to_owned()));
        }
    }

    let content_length = headers
        .iter()
        .find(|(name, _)| name == "content-length")
        .and_then(|(_, value)| value.parse::<usize>().ok())
        .unwrap_or(0);
    if content_length > max_body {
        return Ok(ReadOutcome::PayloadTooLarge);
    }

    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body)?;

    let (path, query) = split_target(target);
    Ok(ReadOutcome::Request(HttpRequest { method: method.to_owned(), path, query, headers, body }))
}

/// Serializes a response with an explicit `Content-Length` and connection close.
fn write_response<W: Write>(writer: &mut W, response: &HttpResponse) -> io::Result<()> {
    let mut head = format!("HTTP/1.1 {} {}\r\n", response.status, reason_phrase(response.status));
    for (name, value) in &response.headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str(&format!("Content-Length: {}\r\n", response.body.len()));
    head.push_str("Connection: close\r\n\r\n");
    writer.write_all(head.as_bytes())?;
    writer.write_all(&response.body)
}

/// Maps the status codes this transport emits to their canonical reason phrases.
fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        413 => "Payload Too Large",
        429 => "Too Many Requests",
        501 => "Not Implemented",
        _ => "Internal Server Error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_request_line_headers_and_body() {
        let raw =
            b"GET /jobs/abc?x=1 HTTP/1.1\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\nhello";
        let mut reader = BufReader::new(&raw[..]);
        let outcome = read_request(&mut reader, 1024).unwrap();
        let ReadOutcome::Request(request) = outcome else {
            panic!("expected a parsed request");
        };
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/jobs/abc");
        assert_eq!(request.query, "x=1");
        assert_eq!(request.header("content-type"), Some("text/plain"));
        assert_eq!(request.body, b"hello");
        assert_eq!(request.query_param("x"), Some("1".to_owned()));
    }

    #[test]
    fn rejects_a_body_that_exceeds_the_limit() {
        let raw = b"POST /jobs HTTP/1.1\r\nContent-Length: 10\r\n\r\n0123456789";
        let mut reader = BufReader::new(&raw[..]);
        assert!(matches!(read_request(&mut reader, 4).unwrap(), ReadOutcome::PayloadTooLarge));
    }

    #[test]
    fn reports_a_clean_close_on_empty_input() {
        let mut reader = BufReader::new(&b""[..]);
        assert!(matches!(read_request(&mut reader, 16).unwrap(), ReadOutcome::Closed));
    }

    #[test]
    fn writes_status_line_length_and_body() {
        let response = HttpResponse::bytes(200, "text/plain", b"hi".to_vec());
        let mut buffer = Vec::new();
        write_response(&mut buffer, &response).unwrap();
        let text = String::from_utf8(buffer).unwrap();
        assert!(text.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(text.contains("Content-Type: text/plain\r\n"));
        assert!(text.contains("Content-Length: 2\r\n"));
        assert!(text.ends_with("\r\n\r\nhi"));
    }
}
