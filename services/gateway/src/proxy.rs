//! The reverse-proxy core: forward a client request to the upstream backend.
//!
//! This is the real proxy leg. It opens a `std::net::TcpStream` to the
//! configured upstream (with a bounded connect timeout), writes the forwarded
//! request line, the hop-by-hop-stripped headers augmented with the
//! `X-Forwarded-*` set, and the body; then it reads and returns the upstream
//! response with hop-by-hop headers stripped from the reply. Idempotent GETs are
//! retried a bounded number of times on connection failure.

use crate::config::GatewayConfig;
use crate::errors::GatewayError;
use crate::headers::{build_forwarded_headers, strip_hop_by_hop};
use crate::http::{self, HttpRequest, HttpResponse};
use crate::retry::RetryPolicy;
use crate::timeouts;
use crate::tls::TlsMode;
use crate::uploads;

/// Forwards `request` to the upstream defined by `config`, on behalf of
/// `client_ip`, and returns the upstream response.
pub fn forward(
    config: &GatewayConfig,
    request: &HttpRequest,
    client_ip: &str,
) -> Result<HttpResponse, GatewayError> {
    let proto = match config.tls_mode {
        TlsMode::TerminatedUpstream => "https",
        TlsMode::Disabled => "http",
    };
    let host = request.header("host");
    let headers = build_forwarded_headers(&request.headers, client_ip, proto, host);
    let policy = RetryPolicy::default();

    let mut attempts = 0u32;
    loop {
        attempts += 1;
        match attempt(config, request, &headers) {
            Ok(response) => return Ok(response),
            Err(error) => {
                if is_connection_failure(&error) && policy.should_retry(&request.method, attempts) {
                    continue;
                }
                return Err(error);
            }
        }
    }
}

/// A single connect-write-read cycle against the upstream.
fn attempt(
    config: &GatewayConfig,
    request: &HttpRequest,
    headers: &[(String, String)],
) -> Result<HttpResponse, GatewayError> {
    let mut stream = timeouts::connect(&config.upstream_addr, config.request_timeout)?;

    http::write_request_head(
        &mut stream,
        &request.method,
        &request.target(),
        headers,
        request.body.len(),
    )
    .map_err(|error| timeouts::classify_io(&error))?;

    // Stream the request body upstream in bounded chunks.
    uploads::passthrough(&mut &request.body[..], &mut stream, config.max_body_bytes)?;

    let response = http::read_response(&mut stream, config.max_body_bytes)?;
    Ok(HttpResponse { headers: strip_hop_by_hop(&response.headers), ..response })
}

/// Whether an error represents a failure to establish the connection, which is
/// safe to retry for an idempotent method.
fn is_connection_failure(error: &GatewayError) -> bool {
    matches!(error, GatewayError::UpstreamUnavailable(_))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    /// Spawns a one-shot mock upstream that returns `canned` after reading the
    /// request, and returns its address plus a receiver for the raw request.
    fn spawn_upstream(canned: &'static [u8]) -> (String, thread::JoinHandle<Vec<u8>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap().to_string();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let received = drain_request(&mut stream);
            stream.write_all(canned).unwrap();
            received
        });
        (address, handle)
    }

    /// Reads a complete request (head plus any `Content-Length` body) so the
    /// mock does not race ahead of the client's body write.
    fn drain_request(stream: &mut TcpStream) -> Vec<u8> {
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 1024];
        let mut header_end = None;
        while header_end.is_none() {
            let read = stream.read(&mut chunk).unwrap();
            if read == 0 {
                return buffer;
            }
            buffer.extend_from_slice(&chunk[..read]);
            header_end = buffer.windows(4).position(|w| w == b"\r\n\r\n").map(|i| i + 4);
        }
        let header_end = header_end.unwrap();
        let head = String::from_utf8_lossy(&buffer[..header_end]).to_ascii_lowercase();
        let content_length = head
            .lines()
            .find_map(|line| line.strip_prefix("content-length:"))
            .and_then(|value| value.trim().parse::<usize>().ok())
            .unwrap_or(0);
        let mut remaining = content_length.saturating_sub(buffer.len() - header_end);
        while remaining > 0 {
            let read = stream.read(&mut chunk).unwrap();
            if read == 0 {
                break;
            }
            buffer.extend_from_slice(&chunk[..read]);
            remaining = remaining.saturating_sub(read);
        }
        buffer
    }

    #[test]
    fn forwards_request_and_returns_upstream_response() {
        let (address, handle) = spawn_upstream(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\nworld",
        );
        let config = GatewayConfig::new("127.0.0.1:0", address);
        let request = HttpRequest::new(
            "POST",
            "/v1/runs",
            vec![("Content-Type".into(), "text/plain".into())],
            b"hello".to_vec(),
        );

        let response = forward(&config, &request, "9.9.9.9").unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body, b"world");

        let received = String::from_utf8(handle.join().unwrap()).unwrap();
        assert!(received.starts_with("POST /v1/runs HTTP/1.1\r\n"));
        assert!(received.contains("X-Forwarded-For: 9.9.9.9"));
        assert!(received.ends_with("hello"));
    }

    #[test]
    fn connection_refused_yields_bad_gateway() {
        let config = GatewayConfig::new("127.0.0.1:0", "127.0.0.1:1");
        let request = HttpRequest::new("POST", "/v1/runs", Vec::new(), Vec::new());
        let error = forward(&config, &request, "1.1.1.1").unwrap_err();
        assert_eq!(error.status(), 502);
    }
}
