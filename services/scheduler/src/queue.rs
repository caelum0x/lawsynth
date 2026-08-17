//! Queued-job ordering and selection.
//!
//! The scheduler holds queued jobs in a map keyed by id; this module turns the
//! set of *eligible* candidates (already filtered by [`crate::placement`]) into a
//! single choice using the priority order defined in [`crate::priority`]. The
//! selection is a pure function of its inputs — no clock, no state — so the
//! dispatch decision is deterministic and testable in isolation.

use crate::priority::Candidate;

/// Selects the highest-priority candidate, or `None` when the queue is empty.
///
/// Highest priority is the *minimum* [`Candidate`] under earliest-deadline-first
/// ordering. Ties are resolved deterministically by submission time then id.
pub fn select(candidates: impl IntoIterator<Item = Candidate>) -> Option<Candidate> {
    candidates.into_iter().min()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_the_earliest_deadline() {
        let chosen = select([
            Candidate::new(300, 0, "c"),
            Candidate::new(100, 0, "a"),
            Candidate::new(200, 0, "b"),
        ])
        .unwrap();
        assert_eq!(chosen.id, "a");
    }

    #[test]
    fn empty_queue_selects_nothing() {
        assert!(select(std::iter::empty()).is_none());
    }

    #[test]
    fn tie_break_is_submission_then_id() {
        let chosen = select([
            Candidate::new(100, 20, "a"),
            Candidate::new(100, 10, "z"),
            Candidate::new(100, 10, "b"),
        ])
        .unwrap();
        assert_eq!((chosen.submitted_at_ms, chosen.id.as_str()), (10, "b"));
    }
}
