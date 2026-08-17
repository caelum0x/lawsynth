//! Cooperative, graceful stop for the accept loop.
//!
//! `std::net::TcpListener::incoming` blocks indefinitely, which makes a clean
//! stop awkward. The gateway instead runs the listener in non-blocking mode and
//! polls a shared flag between accepts, so a [`ShutdownHandle::trigger`] call
//! causes the loop to exit after finishing at most one poll interval — without
//! severing connections that are already being served.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// The server-side view of the shutdown flag, checked inside the accept loop.
#[derive(Clone, Debug, Default)]
pub struct Shutdown {
    flag: Arc<AtomicBool>,
}

impl Shutdown {
    pub fn new() -> Self {
        Self { flag: Arc::new(AtomicBool::new(false)) }
    }

    /// A cloneable handle that can request the accept loop to stop.
    pub fn handle(&self) -> ShutdownHandle {
        ShutdownHandle { flag: Arc::clone(&self.flag) }
    }

    /// True once a stop has been requested.
    pub fn is_triggered(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

/// A detached trigger for requesting graceful shutdown from another thread.
#[derive(Clone, Debug)]
pub struct ShutdownHandle {
    flag: Arc<AtomicBool>,
}

impl ShutdownHandle {
    /// Requests the accept loop to stop accepting new connections.
    pub fn trigger(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_triggers_shared_flag() {
        let shutdown = Shutdown::new();
        assert!(!shutdown.is_triggered());
        let handle = shutdown.handle();
        handle.trigger();
        assert!(shutdown.is_triggered());
    }
}
