use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// Cooperative, thread-safe cancellation shared by bounded engine operations.
#[derive(Clone, Debug, Default)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_state_is_shared_between_clones() {
        let first = CancellationToken::default();
        let second = first.clone();
        first.cancel();
        assert!(second.is_cancelled());
    }
}
