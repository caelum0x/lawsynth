//! A bounded, in-process log of completed request events.
//!
//! Where [`crate::tracing`] renders a single structured line for stderr and
//! [`crate::metrics`] keeps aggregate counters, this module retains the most
//! recent individual request events so an operator (or a `/events` scrape) can
//! inspect recent traffic without an external log pipeline. The log is an
//! append-only ring buffer with a strictly increasing sequence, so a consumer
//! can poll `since(cursor)` and observe every event exactly once until it is
//! evicted by the bound. It links nothing beyond `std`.

use std::collections::VecDeque;
use std::sync::Mutex;

/// One completed request, captured after the response status is known.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestEvent {
    pub sequence: u64,
    pub request_id: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub client_ip: String,
    pub duration_micros: u128,
}

/// A thread-safe, bounded ring buffer of recent [`RequestEvent`]s.
///
/// `capacity` caps memory: once full, appending evicts the oldest event. The
/// `sequence` counter never resets, so eviction is observable as a gap between a
/// consumer's cursor and the first retained event.
#[derive(Debug)]
pub struct EventLog {
    capacity: usize,
    inner: Mutex<Inner>,
}

#[derive(Debug)]
struct Inner {
    next_sequence: u64,
    events: VecDeque<RequestEvent>,
}

impl EventLog {
    /// Creates a log retaining at most `capacity` events (minimum 1).
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            inner: Mutex::new(Inner { next_sequence: 1, events: VecDeque::new() }),
        }
    }

    /// Appends a completed request and returns the assigned sequence number.
    pub fn record(
        &self,
        request_id: impl Into<String>,
        method: impl Into<String>,
        path: impl Into<String>,
        status: u16,
        client_ip: impl Into<String>,
        duration_micros: u128,
    ) -> u64 {
        let mut inner = self.inner.lock().expect("gateway event log poisoned");
        let sequence = inner.next_sequence;
        inner.next_sequence += 1;
        inner.events.push_back(RequestEvent {
            sequence,
            request_id: request_id.into(),
            method: method.into(),
            path: path.into(),
            status,
            client_ip: client_ip.into(),
            duration_micros,
        });
        while inner.events.len() > self.capacity {
            inner.events.pop_front();
        }
        sequence
    }

    /// Returns all retained events with a sequence strictly greater than `cursor`.
    pub fn since(&self, cursor: u64) -> Vec<RequestEvent> {
        let inner = self.inner.lock().expect("gateway event log poisoned");
        inner.events.iter().filter(|event| event.sequence > cursor).cloned().collect()
    }

    /// Returns the number of currently retained events.
    pub fn len(&self) -> usize {
        self.inner.lock().expect("gateway event log poisoned").events.len()
    }

    /// Returns whether the log currently holds no events.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(log: &EventLog, path: &str, status: u16) -> u64 {
        log.record("req-1", "GET", path, status, "127.0.0.1", 1500)
    }

    #[test]
    fn assigns_strictly_increasing_sequences() {
        let log = EventLog::new(8);
        assert_eq!(record(&log, "/v1/runs", 200), 1);
        assert_eq!(record(&log, "/v1/runs", 200), 2);
        assert_eq!(record(&log, "/v1/health", 200), 3);
    }

    #[test]
    fn since_returns_only_newer_events() {
        let log = EventLog::new(8);
        record(&log, "/a", 200);
        record(&log, "/b", 404);
        let tail = log.since(1);
        assert_eq!(tail.len(), 1);
        assert_eq!(tail[0].path, "/b");
        assert_eq!(tail[0].status, 404);
        assert!(log.since(2).is_empty());
    }

    #[test]
    fn ring_buffer_evicts_oldest_but_keeps_sequence() {
        let log = EventLog::new(2);
        record(&log, "/1", 200);
        record(&log, "/2", 200);
        record(&log, "/3", 200); // evicts /1
        assert_eq!(log.len(), 2);
        let retained = log.since(0);
        // The oldest surviving event's sequence is 2, so a cursor at 0 sees a
        // gap (event 1 was evicted) — exactly the observable-eviction contract.
        assert_eq!(retained[0].sequence, 2);
        assert_eq!(retained[1].sequence, 3);
    }
}
