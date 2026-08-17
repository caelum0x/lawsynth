//! Job priority ordering for queue selection.
//!
//! The scheduler dispatches the *most urgent* queued job first. Urgency is an
//! earliest-deadline-first order: the job whose hard deadline arrives soonest
//! wins, with submission time and then job id as deterministic tie-breakers so
//! selection is total and reproducible. Modeling this as an explicit `Ord` type
//! keeps the ordering policy in one place instead of scattered comparison tuples.

use std::cmp::Ordering;

/// A queued job reduced to its ordering key.
///
/// Ordering is earliest-deadline-first, then earliest submission, then id. A
/// *smaller* [`Candidate`] is *higher* priority, so `min` selects the job to run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Candidate {
    pub deadline_at_ms: u64,
    pub submitted_at_ms: u64,
    pub id: String,
}

impl Candidate {
    pub fn new(deadline_at_ms: u64, submitted_at_ms: u64, id: impl Into<String>) -> Self {
        Self { deadline_at_ms, submitted_at_ms, id: id.into() }
    }

    /// The ordering key used for comparison, most-significant field first.
    fn key(&self) -> (u64, u64, &str) {
        (self.deadline_at_ms, self.submitted_at_ms, self.id.as_str())
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key())
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn earlier_deadline_is_higher_priority() {
        let urgent = Candidate::new(100, 10, "a");
        let relaxed = Candidate::new(200, 5, "b");
        assert!(urgent < relaxed);
    }

    #[test]
    fn submission_breaks_deadline_ties() {
        let first = Candidate::new(100, 5, "z");
        let second = Candidate::new(100, 9, "a");
        assert!(first < second);
    }

    #[test]
    fn id_breaks_remaining_ties_deterministically() {
        let a = Candidate::new(100, 5, "a");
        let b = Candidate::new(100, 5, "b");
        assert!(a < b);
    }
}
