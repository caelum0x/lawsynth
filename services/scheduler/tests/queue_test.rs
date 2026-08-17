//! Integration tests for queued-job ordering and selection.

use lawsynth_scheduler::{Candidate, select_next};

#[test]
fn selects_the_earliest_deadline_first() {
    let chosen = select_next([
        Candidate::new(300, 0, "c"),
        Candidate::new(100, 0, "a"),
        Candidate::new(200, 0, "b"),
    ])
    .unwrap();
    assert_eq!(chosen.id, "a");
}

#[test]
fn breaks_deadline_ties_by_submission_then_id() {
    let chosen = select_next([
        Candidate::new(100, 20, "a"),
        Candidate::new(100, 10, "z"),
        Candidate::new(100, 10, "b"),
    ])
    .unwrap();
    assert_eq!((chosen.submitted_at_ms, chosen.id.as_str()), (10, "b"));
}

#[test]
fn an_empty_queue_selects_nothing() {
    assert!(select_next(std::iter::empty()).is_none());
}

#[test]
fn a_single_candidate_is_always_selected() {
    let chosen = select_next([Candidate::new(999, 5, "solo")]).unwrap();
    assert_eq!(chosen.id, "solo");
}

#[test]
fn candidate_ordering_is_total_and_consistent() {
    let mut candidates =
        [Candidate::new(200, 1, "b"), Candidate::new(100, 9, "a"), Candidate::new(100, 1, "a")];
    candidates.sort();
    let order: Vec<(&str, u64)> =
        candidates.iter().map(|c| (c.id.as_str(), c.submitted_at_ms)).collect();
    assert_eq!(order, vec![("a", 1), ("a", 9), ("b", 1)]);
}
