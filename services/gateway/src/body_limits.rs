//! Request body size enforcement.
//!
//! The transport already refuses a `Content-Length` larger than the configured
//! maximum while reading, but this module provides the pure predicate that
//! decides the outcome so it can be reused and unit-tested independently, and so
//! the `413` decision has a single, named home.

/// The result of checking a body length against the configured ceiling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BodyCheck {
    /// The body is within the limit.
    Ok,
    /// The body exceeds the limit; the caller must answer `413`.
    TooLarge { limit: usize, actual: usize },
}

impl BodyCheck {
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }
}

/// Returns whether `actual` bytes are acceptable under `limit`.
pub fn check(actual: usize, limit: usize) -> BodyCheck {
    if actual > limit { BodyCheck::TooLarge { limit, actual } } else { BodyCheck::Ok }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_within_limit() {
        assert!(check(10, 16).is_ok());
        assert!(check(16, 16).is_ok());
    }

    #[test]
    fn rejects_over_limit() {
        assert_eq!(check(17, 16), BodyCheck::TooLarge { limit: 16, actual: 17 });
    }
}
