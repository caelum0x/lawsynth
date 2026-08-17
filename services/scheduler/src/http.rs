//! A dependency-free HTTP/1.1 control-plane transport for the scheduler.
//!
//! The scheduler core links no async runtime or HTTP framework, so this module
//! implements a small, blocking, thread-per-connection server on `std::net`. It
//! parses one request per connection, routes it through [`crate::router`], writes
//! the response, and closes the connection. Time is provided by an injected clock
//! so routing stays deterministic and testable; [`SchedulerServer::with_system_clock`]
//! wires the real wall clock for `main`.
//!
//! CONTROL-PLANE BOUNDARY: this transport serves ONLY the scheduler's
//! serializable control plane (health, pool registration, job state, checkpoints,
//! cancellation, expiry recovery). It never dispatches executable work. The
//! `JobEnvelope` payload is a typed, in-process value with no wire codec (see
//! `lib.rs` and `scheduler.rs`), so lease / heartbeat / complete / fail — which
//! carry or fence executable envelopes — are deliberately absent from the route
//! table and remain in-process API calls only.

use crate::router;
use crate::{Scheduler, SchedulerError};
use lawsynth_store::ObjectStore;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

/// Supplies the current Unix time in milliseconds to time-dependent operations.
///
/// The scheduler reasons in milliseconds (`now_ms`), so the clock returns
/// milliseconds rather than the seconds used by other services.
pub type Clock = Arc<dyn Fn() -> u64 + Send + Sync>;

/// Absolute ceiling on a request line + header block, guarding against a client
/// that never sends the terminating blank line.
const MAX_HEADER_BYTES: usize = 64 * 1024;

/// Ceiling on a control-plane request body. Bodies are shallow JSON objects, so a
/// small bound is generous while still refusing unbounded reads.
const MAX_BODY_BYTES: usize = 64 * 1024;

/// A parsed HTTP request. Header names are stored lowercased for case-insensitive
/// lookup; the body is bounded by [`MAX_BODY_BYTES`].
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
    pub fn json(status: u16, value: &crate::json::Json) -> Self {
        Self::bytes(status, "application/json", value.render().into_bytes())
    }

    /// Maps a domain error to its documented status and machine-readable body.
    pub fn error(error: &SchedulerError) -> Self {
        let classified = crate::http_error::classify(error);
        Self::json(classified.status, &crate::http_error::body(error))
    }

    /// Builds an error response from a transport-level code discovered before a
    /// domain call (malformed method, unmatched route, malformed body).
    pub fn error_code(status: u16, code: &'static str, message: &str) -> Self {
        Self::json(
            status,
            &crate::json::Json::Object(vec![
                ("code".into(), crate::json::Json::string(code)),
                ("message".into(), crate::json::Json::string(message)),
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

/// A blocking HTTP transport bound to a shared, mutable [`Scheduler`].
///
/// The scheduler owns mutable lifecycle state and its operations take `&mut self`,
/// so it is shared behind an `Arc<Mutex<_>>`: each connection thread briefly locks
/// the scheduler to route a single request. This serializes control-plane
/// mutations, which is exactly the deterministic contract the core provides.
pub struct SchedulerServer<S> {
    scheduler: Arc<Mutex<Scheduler<S>>>,
    clock: Clock,
}

impl<S> Clone for SchedulerServer<S> {
    fn clone(&self) -> Self {
        Self { scheduler: Arc::clone(&self.scheduler), clock: Arc::clone(&self.clock) }
    }
}

impl<S: ObjectStore> SchedulerServer<S> {
    /// Builds a server with an explicit clock; tests inject a fixed value here.
    pub fn new(scheduler: Arc<Mutex<Scheduler<S>>>, clock: Clock) -> Self {
        Self { scheduler, clock }
    }

    /// Builds a server whose clock reads the system wall clock in Unix milliseconds.
    pub fn with_system_clock(scheduler: Arc<Mutex<Scheduler<S>>>) -> Self {
        let clock: Clock = Arc::new(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|elapsed| elapsed.as_millis().try_into().unwrap_or(u64::MAX))
                .unwrap_or(0)
        });
        Self::new(scheduler, clock)
    }

    /// Routes an already-parsed request, used by tests that bypass sockets.
    ///
    /// A poisoned lock is recovered rather than propagated: one panicking
    /// connection thread must not wedge the whole control plane.
    pub fn handle(&self, request: &HttpRequest) -> HttpResponse {
        let now = (self.clock)();
        let mut scheduler = self.scheduler.lock().unwrap_or_else(|poison| poison.into_inner());
        router::route(&mut scheduler, now, request)
    }

    /// Serves connections until the listener errors, one thread per connection.
    pub fn serve(&self, listener: &TcpListener) -> io::Result<()>
    where
        S: Send + 'static,
    {
        for stream in listener.incoming() {
            let stream = stream?;
            let worker = self.clone();
            thread::spawn(move || {
                if let Err(error) = worker.serve_connection(stream) {
                    eprintln!("lawsynth-scheduler: connection error: {error}");
                }
            });
        }
        Ok(())
    }

    fn serve_connection(&self, stream: TcpStream) -> io::Result<()> {
        let mut reader = BufReader::new(stream.try_clone()?);
        let mut writer = stream;
        let response = match read_request(&mut reader)? {
            ReadOutcome::Closed => return Ok(()),
            ReadOutcome::PayloadTooLarge => HttpResponse::error_code(
                413,
                "payload_too_large",
                "request exceeds the configured maximum",
            ),
            ReadOutcome::Request(request) => self.handle(&request),
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
fn read_request<R: BufRead>(reader: &mut R) -> io::Result<ReadOutcome> {
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
    if content_length > MAX_BODY_BYTES {
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
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        413 => "Payload Too Large",
        501 => "Not Implemented",
        _ => "Internal Server Error",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_request_line_headers_and_body() {
        let raw = b"POST /pools?x=1 HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: 5\r\n\r\nhello";
        let mut reader = BufReader::new(&raw[..]);
        let outcome = read_request(&mut reader).unwrap();
        let ReadOutcome::Request(request) = outcome else {
            panic!("expected a parsed request");
        };
        assert_eq!(request.method, "POST");
        assert_eq!(request.path, "/pools");
        assert_eq!(request.query, "x=1");
        assert_eq!(request.header("content-type"), Some("application/json"));
        assert_eq!(request.body, b"hello");
        assert_eq!(request.query_param("x"), Some("1".to_owned()));
    }

    #[test]
    fn rejects_a_body_that_exceeds_the_limit() {
        let oversized = MAX_BODY_BYTES + 1;
        let raw = format!("POST /pools HTTP/1.1\r\nContent-Length: {oversized}\r\n\r\n");
        let mut reader = BufReader::new(raw.as_bytes());
        assert!(matches!(read_request(&mut reader).unwrap(), ReadOutcome::PayloadTooLarge));
    }

    #[test]
    fn reports_a_clean_close_on_empty_input() {
        let mut reader = BufReader::new(&b""[..]);
        assert!(matches!(read_request(&mut reader).unwrap(), ReadOutcome::Closed));
    }

    #[test]
    fn writes_status_line_length_and_body() {
        let response = HttpResponse::bytes(200, "application/json", b"{}".to_vec());
        let mut buffer = Vec::new();
        write_response(&mut buffer, &response).unwrap();
        let text = String::from_utf8(buffer).unwrap();
        assert!(text.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(text.contains("Content-Type: application/json\r\n"));
        assert!(text.contains("Content-Length: 2\r\n"));
        assert!(text.ends_with("\r\n\r\n{}"));
    }
}
