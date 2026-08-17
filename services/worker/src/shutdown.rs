//! Graceful drain coordination.
//!
//! A worker asked to stop must refuse new work while letting in-flight jobs run
//! to a clean terminal state, then report itself drained. This controller models
//! that lifecycle: it is `Accepting` until a drain begins, `Draining` while work
//! is still in flight, and `Drained` once the last in-flight unit leaves. It is
//! thread-safe so the shared, `Arc`-held worker can consult it from any
//! connection thread.

use std::sync::Mutex;

use crate::WorkerError;

/// The controller's lifecycle phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrainState {
    /// New work is admitted.
    Accepting,
    /// A drain has begun; new work is refused but jobs remain in flight.
    Draining,
    /// A drain has begun and no work remains in flight.
    Drained,
}

#[derive(Debug)]
struct Inner {
    draining: bool,
    in_flight: u64,
}

/// Coordinates graceful shutdown by gating admission and tracking in-flight work.
#[derive(Debug)]
pub struct ShutdownController {
    inner: Mutex<Inner>,
}

impl Default for ShutdownController {
    fn default() -> Self {
        Self::new()
    }
}

impl ShutdownController {
    pub fn new() -> Self {
        Self { inner: Mutex::new(Inner { draining: false, in_flight: 0 }) }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().expect("shutdown controller mutex poisoned")
    }

    /// The current lifecycle phase.
    pub fn state(&self) -> DrainState {
        let inner = self.lock();
        match (inner.draining, inner.in_flight) {
            (false, _) => DrainState::Accepting,
            (true, 0) => DrainState::Drained,
            (true, _) => DrainState::Draining,
        }
    }

    /// The number of admitted units of work that have not yet left.
    pub fn in_flight(&self) -> u64 {
        self.lock().in_flight
    }

    /// Whether the controller has begun draining and all work has left.
    pub fn is_drained(&self) -> bool {
        matches!(self.state(), DrainState::Drained)
    }

    /// Begins draining. Idempotent: repeated calls have no additional effect.
    pub fn begin_drain(&self) {
        self.lock().draining = true;
    }

    /// Admits one unit of work, returning a guard that releases it on drop. Once
    /// draining has begun, admission is refused so in-flight work can complete
    /// without new arrivals.
    pub fn enter(&self) -> Result<WorkGuard<'_>, WorkerError> {
        let mut inner = self.lock();
        if inner.draining {
            return Err(WorkerError::LimitExceeded(
                "worker is draining and is not admitting new work".into(),
            ));
        }
        inner.in_flight += 1;
        Ok(WorkGuard { controller: self })
    }

    fn leave(&self) {
        let mut inner = self.lock();
        inner.in_flight = inner.in_flight.saturating_sub(1);
    }
}

/// An RAII marker for one admitted unit of work; dropping it decrements the
/// in-flight count, so drain accounting cannot leak on early returns.
#[derive(Debug)]
pub struct WorkGuard<'a> {
    controller: &'a ShutdownController,
}

impl Drop for WorkGuard<'_> {
    fn drop(&mut self) {
        self.controller.leave();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transitions_from_accepting_through_draining_to_drained() {
        let controller = ShutdownController::new();
        assert_eq!(controller.state(), DrainState::Accepting);

        let guard = controller.enter().expect("accepting admits work");
        assert_eq!(controller.in_flight(), 1);

        controller.begin_drain();
        assert_eq!(controller.state(), DrainState::Draining);
        assert!(controller.enter().is_err(), "draining refuses new work");

        drop(guard);
        assert_eq!(controller.state(), DrainState::Drained);
        assert!(controller.is_drained());
    }

    #[test]
    fn drain_with_no_in_flight_work_is_immediately_drained() {
        let controller = ShutdownController::default();
        controller.begin_drain();
        assert!(controller.is_drained());
    }
}
