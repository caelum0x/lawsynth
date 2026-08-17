//! Deterministic request identifiers and structured request logging.
//!
//! Request IDs are drawn from a monotonic counter rather than a random source so
//! that behaviour is reproducible in tests and across replays. IDs are unique
//! within a process lifetime, which is all a correlation identifier needs.

use std::sync::atomic::{AtomicU64, Ordering};

/// A monotonic, thread-safe source of request identifiers.
#[derive(Debug, Default)]
pub struct RequestIds {
    counter: AtomicU64,
}

impl RequestIds {
    pub fn new() -> Self {
        Self { counter: AtomicU64::new(0) }
    }

    /// Returns the next identifier, formatted as a fixed-width hex token.
    ///
    /// The counter starts at zero, so the first ID is `req-0000000000000001`.
    pub fn next_id(&self) -> String {
        let value = self.counter.fetch_add(1, Ordering::Relaxed) + 1;
        format!("req-{value:016x}")
    }
}

/// A single structured log line describing a completed request.
///
/// Returned as a string (rather than written directly) so tests can assert on
/// its exact shape; the server prints it to stderr.
pub fn request_log_line(
    request_id: &str,
    method: &str,
    path: &str,
    status: u16,
    client_ip: &str,
    duration_micros: u128,
) -> String {
    format!(
        "request_id={request_id} method={method} path={path} status={status} client={client_ip} duration_us={duration_micros}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_increment_monotonically() {
        let ids = RequestIds::new();
        assert_eq!(ids.next_id(), "req-0000000000000001");
        assert_eq!(ids.next_id(), "req-0000000000000002");
    }

    #[test]
    fn log_line_carries_all_fields() {
        let line = request_log_line("req-1", "GET", "/v1/runs", 200, "127.0.0.1", 1500);
        assert!(line.contains("request_id=req-1"));
        assert!(line.contains("method=GET"));
        assert!(line.contains("path=/v1/runs"));
        assert!(line.contains("status=200"));
        assert!(line.contains("client=127.0.0.1"));
        assert!(line.contains("duration_us=1500"));
    }
}
