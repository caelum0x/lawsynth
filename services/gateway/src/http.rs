//! A dependency-free HTTP/1.1 transport for the gateway.
//!
//! Like the artifact service, the gateway links no async runtime or HTTP
//! framework. This module owns both legs of the proxy: the *server* leg parses
//! one client request per connection (bounded by the configured limits), and the
//! *client* leg serializes a forwarded request to the upstream and parses the
//! upstream response. Both legs are plain blocking `std::io` over `std::net`.

use crate::errors::GatewayError;
use crate::json::Json;
use std::io::{self, BufRead, Read, Write};

/// A parsed HTTP request. Header names are lowercased for case-insensitive
/// lookup; header order is preserved for faithful forwarding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    /// Constructs a request directly, primarily for routing and proxy tests.
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

    /// Reconstructs the origin-form request target (`path` plus optional query).
    pub fn target(&self) -> String {
        if self.query.is_empty() {
            self.path.clone()
        } else {
            format!("{}?{}", self.path, self.query)
        }
    }
}

/// An HTTP response ready to be written to a client, or parsed from an upstream.
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

    /// Returns the first value for a response header, matched case-insensitively.
    pub fn header(&self, name: &str) -> Option<&str> {
        let name = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(&name))
            .map(|(_, value)| value.as_str())
    }

    /// Body of raw bytes served with an explicit content type.
    pub fn bytes(status: u16, content_type: &str, body: Vec<u8>) -> Self {
        Self::new(status).with_header("Content-Type", content_type).body(body)
    }

    /// Renders a JSON value as an `application/json` body.
    pub fn json(status: u16, value: &Json) -> Self {
        Self::bytes(status, "application/json", value.render().into_bytes())
    }

    /// Builds a machine-readable error envelope with a stable code and message.
    pub fn error_code(status: u16, code: &str, message: &str) -> Self {
        Self::json(
            status,
            &Json::Object(vec![
                ("code".into(), Json::string(code)),
                ("message".into(), Json::string(message)),
            ]),
        )
    }

    /// Maps a gateway error to its documented status and error envelope.
    pub fn error(error: &GatewayError) -> Self {
        Self::error_code(error.status(), error.code(), &error.to_string())
    }

    fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }
}

/// Outcome of reading a single request from a client connection.
pub enum ReadOutcome {
    Request(HttpRequest),
    PayloadTooLarge,
    HeaderFieldsTooLarge,
    Closed,
}

/// Splits a request target into its path and raw query components.
pub fn split_target(target: &str) -> (String, String) {
    match target.split_once('?') {
        Some((path, query)) => (path.to_owned(), query.to_owned()),
        None => (target.to_owned(), String::new()),
    }
}

/// Reads and parses a single client request, enforcing header/body ceilings.
pub fn read_request<R: BufRead>(
    reader: &mut R,
    max_body: usize,
    max_header_bytes: usize,
    max_headers: usize,
) -> io::Result<ReadOutcome> {
    let mut header_block = Vec::new();
    loop {
        let mut line = Vec::new();
        let read = reader.read_until(b'\n', &mut line)?;
        if read == 0 {
            return Ok(ReadOutcome::Closed);
        }
        if line == b"\r\n" || line == b"\n" {
            break;
        }
        header_block.extend_from_slice(&line);
        if header_block.len() > max_header_bytes {
            return Ok(ReadOutcome::HeaderFieldsTooLarge);
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
            if headers.len() > max_headers {
                return Ok(ReadOutcome::HeaderFieldsTooLarge);
            }
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

/// Serializes a response to a client with explicit `Content-Length` and close.
pub fn write_response<W: Write>(writer: &mut W, response: &HttpResponse) -> io::Result<()> {
    let mut head = format!("HTTP/1.1 {} {}\r\n", response.status, reason_phrase(response.status));
    for (name, value) in &response.headers {
        if name.eq_ignore_ascii_case("content-length") || name.eq_ignore_ascii_case("connection") {
            continue;
        }
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str(&format!("Content-Length: {}\r\n", response.body.len()));
    head.push_str("Connection: close\r\n\r\n");
    writer.write_all(head.as_bytes())?;
    writer.write_all(&response.body)
}

/// Serializes the forwarded request line and headers onto the upstream socket.
///
/// The body is written separately (streamed by `crate::uploads`) so large upload
/// bodies never need to be re-buffered here. `body_len` becomes the framed
/// `Content-Length`, and `Connection: close` lets the response be read to EOF.
pub fn write_request_head<W: Write>(
    writer: &mut W,
    method: &str,
    target: &str,
    headers: &[(String, String)],
    body_len: usize,
) -> io::Result<()> {
    let mut head = format!("{method} {target} HTTP/1.1\r\n");
    for (name, value) in headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str(&format!("Content-Length: {body_len}\r\n"));
    head.push_str("Connection: close\r\n\r\n");
    writer.write_all(head.as_bytes())
}

/// Reads and parses the upstream response, bounding the buffered body.
///
/// The gateway always sends `Connection: close` upstream, so when no
/// `Content-Length` is advertised the body is the remainder of the stream up to
/// `max_body`. A `Content-Length` larger than `max_body` is rejected.
pub fn read_response<R: Read>(
    reader: &mut R,
    max_body: usize,
) -> Result<HttpResponse, GatewayError> {
    let mut buffered = io::BufReader::new(reader);
    let mut status_line = String::new();
    read_line(&mut buffered, &mut status_line)?;
    if status_line.is_empty() {
        return Err(GatewayError::BadUpstreamResponse("empty status line".into()));
    }
    let status = parse_status(&status_line)?;

    let mut headers = Vec::new();
    loop {
        let mut line = String::new();
        read_line(&mut buffered, &mut line)?;
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':') {
            headers.push((name.trim().to_owned(), value.trim().to_owned()));
        }
    }

    let declared = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, value)| value.trim().parse::<usize>().ok());

    let body = match declared {
        Some(length) => {
            if length > max_body {
                return Err(GatewayError::PayloadTooLarge);
            }
            let mut body = vec![0u8; length];
            buffered
                .read_exact(&mut body)
                .map_err(|error| GatewayError::BadUpstreamResponse(error.to_string()))?;
            body
        }
        None => {
            // No declared length: the upstream signals end-of-body by closing.
            // Copy it in bounded chunks so a large artifact never balloons memory.
            let mut body = Vec::new();
            crate::downloads::stream_copy(&mut buffered, &mut body, max_body)?;
            body
        }
    };

    Ok(HttpResponse { status, headers, body })
}

fn read_line<R: BufRead>(reader: &mut R, out: &mut String) -> Result<(), GatewayError> {
    reader.read_line(out).map_err(|error| GatewayError::BadUpstreamResponse(error.to_string()))?;
    Ok(())
}

fn parse_status(status_line: &str) -> Result<u16, GatewayError> {
    let mut parts = status_line.split_whitespace();
    let _version =
        parts.next().ok_or_else(|| GatewayError::BadUpstreamResponse("missing version".into()))?;
    let code = parts
        .next()
        .ok_or_else(|| GatewayError::BadUpstreamResponse("missing status code".into()))?;
    code.parse::<u16>()
        .map_err(|_| GatewayError::BadUpstreamResponse(format!("non-numeric status: {code}")))
}

/// Maps status codes the gateway emits to their canonical reason phrases.
pub fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        413 => "Payload Too Large",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "OK",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    #[test]
    fn parses_request_line_headers_and_body() {
        let raw = b"POST /v1/runs?x=1 HTTP/1.1\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\nhello";
        let mut reader = BufReader::new(&raw[..]);
        let ReadOutcome::Request(request) = read_request(&mut reader, 1024, 8192, 32).unwrap()
        else {
            panic!("expected a parsed request");
        };
        assert_eq!(request.method, "POST");
        assert_eq!(request.path, "/v1/runs");
        assert_eq!(request.query, "x=1");
        assert_eq!(request.target(), "/v1/runs?x=1");
        assert_eq!(request.header("content-type"), Some("text/plain"));
        assert_eq!(request.body, b"hello");
    }

    #[test]
    fn rejects_a_body_over_the_limit() {
        let raw = b"POST /v1/runs HTTP/1.1\r\nContent-Length: 10\r\n\r\n0123456789";
        let mut reader = BufReader::new(&raw[..]);
        assert!(matches!(
            read_request(&mut reader, 4, 8192, 32).unwrap(),
            ReadOutcome::PayloadTooLarge
        ));
    }

    #[test]
    fn rejects_too_many_headers() {
        let raw = b"GET /v1/x HTTP/1.1\r\nA: 1\r\nB: 2\r\nC: 3\r\n\r\n";
        let mut reader = BufReader::new(&raw[..]);
        assert!(matches!(
            read_request(&mut reader, 16, 8192, 2).unwrap(),
            ReadOutcome::HeaderFieldsTooLarge
        ));
    }

    #[test]
    fn parses_upstream_response_with_content_length() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n\r\nhi";
        let response = read_response(&mut &raw[..], 1024).unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body, b"hi");
        assert_eq!(response.header("content-type"), Some("text/plain"));
    }

    #[test]
    fn parses_upstream_response_read_to_close() {
        let raw = b"HTTP/1.1 201 Created\r\nX-Marker: yes\r\n\r\npayload";
        let response = read_response(&mut &raw[..], 1024).unwrap();
        assert_eq!(response.status, 201);
        assert_eq!(response.body, b"payload");
    }

    #[test]
    fn rejects_oversized_upstream_body() {
        let raw = b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n";
        let error = read_response(&mut &raw[..], 4).unwrap_err();
        assert_eq!(error.status(), 413);
    }

    #[test]
    fn writes_status_line_length_and_body() {
        let response = HttpResponse::bytes(200, "text/plain", b"hi".to_vec());
        let mut buffer = Vec::new();
        write_response(&mut buffer, &response).unwrap();
        let text = String::from_utf8(buffer).unwrap();
        assert!(text.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(text.contains("Content-Length: 2\r\n"));
        assert!(text.ends_with("\r\n\r\nhi"));
    }
}
