//! Bounded retry policy for idempotent upstream requests.
//!
//! Only methods defined as idempotent by HTTP semantics (RFC 7231 §4.2.2) may be
//! retried, and only on *connection-establishment* failures — never after any
//! bytes of a request have been accepted by the upstream, which could duplicate
//! a side effect. The policy is a pure decision function so it is trivial to
//! test; `proxy` drives the actual retry loop with it.

/// Retry configuration: how many total attempts an eligible request may make.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetryPolicy {
    /// Total attempts including the first. `1` disables retries.
    pub max_attempts: u32,
}

impl RetryPolicy {
    pub const fn new(max_attempts: u32) -> Self {
        Self { max_attempts: if max_attempts == 0 { 1 } else { max_attempts } }
    }

    /// Whether the given method is safe to retry on a connection failure.
    pub fn is_idempotent(method: &str) -> bool {
        matches!(
            method.to_ascii_uppercase().as_str(),
            "GET" | "HEAD" | "OPTIONS" | "PUT" | "DELETE"
        )
    }

    /// Whether another attempt is permitted for `method` after `attempts_made`.
    ///
    /// `attempts_made` counts attempts already performed (starting at 1 after the
    /// first try). Non-idempotent methods are never retried.
    pub fn should_retry(&self, method: &str, attempts_made: u32) -> bool {
        Self::is_idempotent(method) && attempts_made < self.max_attempts
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_is_retried_until_the_cap() {
        let policy = RetryPolicy::new(3);
        assert!(policy.should_retry("GET", 1));
        assert!(policy.should_retry("GET", 2));
        assert!(!policy.should_retry("GET", 3));
    }

    #[test]
    fn post_is_never_retried() {
        let policy = RetryPolicy::new(3);
        assert!(!policy.should_retry("POST", 1));
    }

    #[test]
    fn zero_attempts_is_clamped_to_one() {
        let policy = RetryPolicy::new(0);
        assert_eq!(policy.max_attempts, 1);
        assert!(!policy.should_retry("GET", 1));
    }
}
