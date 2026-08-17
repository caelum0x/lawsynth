//! Multi-objective Pareto frontier construction over discovery candidates.
//!
//! Implements architecture §8 ("Pareto frontier construction") and §16
//! ("candidate score vector"). The frontier is the non-dominated set over a
//! score vector drawn from the real `lawsynth-score` objectives:
//!
//! - `error` — mean squared error (minimized);
//! - `complexity` — expression node complexity (minimized);
//! - `stability` — bootstrap selection stability in `[0, 1]` (maximized).
//!
//! Comparisons use [`f64::total_cmp`] so dominance is a total, deterministic
//! order even in the presence of `NaN`, and the frontier is returned in
//! ascending index order for reproducibility.

use std::cmp::Ordering;

use crate::DiscoveryCandidate;

/// The §16 multi-objective score vector for a single candidate.
///
/// `error` and `complexity` are minimization objectives; `stability` is a
/// maximization objective.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CandidateScore {
    pub error: f64,
    pub complexity: usize,
    pub stability: f64,
}

impl CandidateScore {
    /// Returns `true` when `self` Pareto-dominates `other`: no worse on every
    /// objective and strictly better on at least one.
    pub fn dominates(&self, other: &Self) -> bool {
        let no_worse = total_le(self.error, other.error)
            && self.complexity <= other.complexity
            && total_le(other.stability, self.stability);
        let strictly_better = total_lt(self.error, other.error)
            || self.complexity < other.complexity
            || total_lt(other.stability, self.stability);
        no_worse && strictly_better
    }
}

/// Returns the indices of the non-dominated candidates in ascending order.
///
/// Deterministic tie-breaking: candidates with mutually non-dominating (or
/// identical) score vectors are all retained, and ordering follows the input
/// index. A candidate dominated by any other is excluded.
pub fn pareto_frontier(candidates: &[DiscoveryCandidate]) -> Vec<usize> {
    let scores = candidates.iter().map(DiscoveryCandidate::score).collect::<Vec<_>>();
    frontier_of(&scores)
}

/// Frontier over pre-computed score vectors, useful for testing dominance in
/// isolation from world construction.
pub fn frontier_of(scores: &[CandidateScore]) -> Vec<usize> {
    scores
        .iter()
        .enumerate()
        .filter_map(|(index, score)| {
            let dominated = scores
                .iter()
                .enumerate()
                .any(|(other, candidate)| other != index && candidate.dominates(score));
            (!dominated).then_some(index)
        })
        .collect()
}

fn total_le(left: f64, right: f64) -> bool {
    !matches!(left.total_cmp(&right), Ordering::Greater)
}

fn total_lt(left: f64, right: f64) -> bool {
    matches!(left.total_cmp(&right), Ordering::Less)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn score(error: f64, complexity: usize, stability: f64) -> CandidateScore {
        CandidateScore { error, complexity, stability }
    }

    #[test]
    fn strictly_dominated_candidate_is_excluded() {
        // Candidate 1 is worse on error and complexity and no better on
        // stability, so candidate 0 dominates it.
        let scores = [score(0.1, 2, 0.9), score(0.5, 4, 0.9)];
        assert!(scores[0].dominates(&scores[1]));
        assert!(!scores[1].dominates(&scores[0]));
        assert_eq!(frontier_of(&scores), vec![0]);
    }

    #[test]
    fn incomparable_candidates_are_all_retained() {
        // Lower error but higher complexity vs. higher error but lower
        // complexity: neither dominates, both stay on the frontier.
        let scores = [score(0.1, 6, 0.5), score(0.4, 2, 0.5)];
        assert!(!scores[0].dominates(&scores[1]));
        assert!(!scores[1].dominates(&scores[0]));
        assert_eq!(frontier_of(&scores), vec![0, 1]);
    }

    #[test]
    fn stability_is_a_maximization_objective() {
        // Identical error and complexity; higher stability dominates.
        let scores = [score(0.2, 3, 0.95), score(0.2, 3, 0.40)];
        assert!(scores[0].dominates(&scores[1]));
        assert_eq!(frontier_of(&scores), vec![0]);
    }

    #[test]
    fn equal_score_vectors_are_both_kept() {
        let scores = [score(0.2, 3, 0.8), score(0.2, 3, 0.8)];
        assert!(!scores[0].dominates(&scores[1]));
        assert_eq!(frontier_of(&scores), vec![0, 1]);
    }

    #[test]
    fn frontier_mixes_dominated_and_non_dominated_members() {
        let scores = [
            score(0.1, 5, 0.7), // low error, high complexity — frontier
            score(0.6, 2, 0.7), // high error, low complexity — frontier
            score(0.6, 5, 0.7), // dominated by both 0 and 1
            score(0.3, 3, 0.9), // strong balanced candidate — frontier
        ];
        assert_eq!(frontier_of(&scores), vec![0, 1, 3]);
    }
}
