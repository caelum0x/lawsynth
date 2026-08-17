//! Real, in-process request counters exposed at `/metrics`.
//!
//! The gateway tracks the total number of requests, a breakdown by response
//! status, and how many requests were rejected by the rate limiter. Counters are
//! plain atomics plus a small mutex-guarded map, so a snapshot is cheap and
//! consistent enough for a Prometheus-style text scrape.

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

/// Thread-safe counters shared across every connection worker.
#[derive(Debug, Default)]
pub struct Metrics {
    total: AtomicU64,
    rate_limited: AtomicU64,
    by_status: Mutex<BTreeMap<u16, u64>>,
}

impl Metrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Records one completed request and its final response status.
    pub fn record(&self, status: u16) {
        self.total.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut map) = self.by_status.lock() {
            *map.entry(status).or_insert(0) += 1;
        }
    }

    /// Records that a request was rejected by the rate limiter (a `429`).
    pub fn record_rate_limited(&self) {
        self.rate_limited.fetch_add(1, Ordering::Relaxed);
    }

    /// Captures an immutable point-in-time view of all counters.
    pub fn snapshot(&self) -> MetricsSnapshot {
        let by_status = self.by_status.lock().map(|map| map.clone()).unwrap_or_default();
        MetricsSnapshot {
            total: self.total.load(Ordering::Relaxed),
            rate_limited: self.rate_limited.load(Ordering::Relaxed),
            by_status,
        }
    }
}

/// A consistent copy of the counters, safe to render without holding locks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetricsSnapshot {
    pub total: u64,
    pub rate_limited: u64,
    pub by_status: BTreeMap<u16, u64>,
}

impl MetricsSnapshot {
    /// Renders the snapshot as Prometheus-style plain text.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str("# gateway request metrics\n");
        out.push_str(&format!("gateway_requests_total {}\n", self.total));
        out.push_str(&format!("gateway_requests_rate_limited_total {}\n", self.rate_limited));
        for (status, count) in &self.by_status {
            out.push_str(&format!("gateway_responses_total{{status=\"{status}\"}} {count}\n"));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_totals_and_status_breakdown() {
        let metrics = Metrics::new();
        metrics.record(200);
        metrics.record(200);
        metrics.record(404);
        metrics.record_rate_limited();
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.total, 3);
        assert_eq!(snapshot.rate_limited, 1);
        assert_eq!(snapshot.by_status.get(&200), Some(&2));
        assert_eq!(snapshot.by_status.get(&404), Some(&1));
    }

    #[test]
    fn renders_plain_text() {
        let metrics = Metrics::new();
        metrics.record(200);
        let text = metrics.snapshot().render();
        assert!(text.contains("gateway_requests_total 1"));
        assert!(text.contains("gateway_responses_total{status=\"200\"} 1"));
    }
}
