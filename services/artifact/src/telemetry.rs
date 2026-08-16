use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Lock-free local operational counters; no events or payloads leave the process.
#[derive(Clone, Debug, Default)]
pub struct Telemetry {
    uploads: Arc<AtomicU64>,
    downloads: Arc<AtomicU64>,
    checksum_failures: Arc<AtomicU64>,
    gc_deletions: Arc<AtomicU64>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TelemetrySnapshot {
    pub uploads: u64,
    pub downloads: u64,
    pub checksum_failures: u64,
    pub gc_deletions: u64,
}

impl Telemetry {
    pub fn snapshot(&self) -> TelemetrySnapshot {
        TelemetrySnapshot {
            uploads: self.uploads.load(Ordering::Relaxed),
            downloads: self.downloads.load(Ordering::Relaxed),
            checksum_failures: self.checksum_failures.load(Ordering::Relaxed),
            gc_deletions: self.gc_deletions.load(Ordering::Relaxed),
        }
    }
    pub(crate) fn upload(&self) {
        self.uploads.fetch_add(1, Ordering::Relaxed);
    }
    pub(crate) fn download(&self) {
        self.downloads.fetch_add(1, Ordering::Relaxed);
    }
    pub(crate) fn checksum_failure(&self) {
        self.checksum_failures.fetch_add(1, Ordering::Relaxed);
    }
    pub(crate) fn gc_deleted(&self, count: u64) {
        self.gc_deletions.fetch_add(count, Ordering::Relaxed);
    }
}
