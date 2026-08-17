//! The scheduler error taxonomy, re-exported under the plural module name.
//!
//! [`crate::error`] is the single source of truth for [`SchedulerError`]: its
//! variants, `Display` rendering, `std::error::Error` source chain, and the
//! `From<StoreError>` conversion all live there. This module deliberately adds no
//! second definition — it only re-exports that taxonomy so callers can refer to
//! `crate::errors` without introducing a duplicate error type. The transport
//! status/code mapping for these errors lives in [`crate::http_error`].

pub use crate::error::SchedulerError;

/// Convenience alias for fallible scheduler operations.
pub type SchedulerResult<T> = Result<T, SchedulerError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn re_exports_the_single_error_taxonomy() {
        // The alias resolves to the canonical error type defined in `crate::error`.
        let error: SchedulerError = SchedulerError::UnknownJob("job-1".into());
        assert!(error.to_string().contains("job-1"));
        // `SchedulerResult` is the crate's fallible alias over that same type.
        let ok: SchedulerResult<u8> = Ok(7);
        assert_eq!(ok.ok(), Some(7));
    }
}
