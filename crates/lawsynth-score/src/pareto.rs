use crate::CandidateMetrics;

/// Returns indices of non-dominated candidates, preserving input order.
///
/// This is the general all-pairs filter: it works for any number of objectives
/// but costs `O(n^2)` domination checks. For the common two-objective case
/// (error x complexity) prefer [`pareto_front_2d`], which returns the identical
/// front in `O(n log n)`.
pub fn pareto_front(metrics: &[CandidateMetrics]) -> Vec<usize> {
    metrics
        .iter()
        .enumerate()
        .filter_map(|(index, metric)| {
            (!metrics
                .iter()
                .enumerate()
                .any(|(other_index, other)| other_index != index && other.dominates(*metric)))
            .then_some(index)
        })
        .collect()
}

/// Returns indices of non-dominated candidates for the two-objective case,
/// preserving input order, in `O(n log n)` time.
///
/// # Algorithm
///
/// Sort the candidates once by `(mean_squared_error, complexity)` ascending,
/// then sweep left to right maintaining `prefix_min_complexity`, the smallest
/// complexity seen among all *strictly smaller* error values. Points sharing an
/// error value form a group whose minimum complexity is its first (already
/// sorted) member. Under the weak-domination rule in
/// [`CandidateMetrics::dominates`], a candidate survives iff:
///
/// * its complexity equals its group minimum (no same-error point beats it on
///   complexity), **and**
/// * that group minimum is strictly below `prefix_min_complexity` (no
///   smaller-error point ties or beats it on complexity).
///
/// Ties and exact duplicates are preserved exactly as the `O(n^2)`
/// [`pareto_front`] preserves them: identical points never dominate each other,
/// so every copy that clears the sweep is kept. The returned indices are sorted
/// ascending to match [`pareto_front`]'s input order.
pub fn pareto_front_2d(metrics: &[CandidateMetrics]) -> Vec<usize> {
    if metrics.is_empty() {
        return Vec::new();
    }

    let mut order: Vec<usize> = (0..metrics.len()).collect();
    order.sort_by(|&left, &right| {
        metrics[left]
            .mean_squared_error
            .total_cmp(&metrics[right].mean_squared_error)
            .then_with(|| metrics[left].complexity.cmp(&metrics[right].complexity))
            .then_with(|| left.cmp(&right))
    });

    let mut front = Vec::new();
    let mut prefix_min_complexity: Option<usize> = None;
    let mut cursor = 0usize;
    while cursor < order.len() {
        // Gather the group of candidates sharing this error value. The sort
        // above lists them in ascending complexity, so the first is the min.
        let group_error = metrics[order[cursor]].mean_squared_error;
        let group_start = cursor;
        while cursor < order.len() && metrics[order[cursor]].mean_squared_error == group_error {
            cursor += 1;
        }
        let group_min_complexity = metrics[order[group_start]].complexity;

        // The group min is non-dominated only if no strictly-smaller-error
        // candidate reaches its complexity.
        let group_is_open = prefix_min_complexity.is_none_or(|prior| group_min_complexity < prior);
        if group_is_open {
            for &index in &order[group_start..cursor] {
                if metrics[index].complexity == group_min_complexity {
                    front.push(index);
                }
            }
        }

        prefix_min_complexity = Some(match prefix_min_complexity {
            Some(prior) => prior.min(group_min_complexity),
            None => group_min_complexity,
        });
    }

    front.sort_unstable();
    front
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metric(mean_squared_error: f64, complexity: usize) -> CandidateMetrics {
        CandidateMetrics { mean_squared_error, complexity }
    }

    #[test]
    fn removes_dominated_candidates() {
        let front = pareto_front(&[metric(1.0, 1), metric(2.0, 2), metric(0.5, 3)]);
        assert_eq!(front, vec![0, 2]);
    }

    /// Deterministic linear-congruential generator so the equality proof runs
    /// on varied-but-reproducible inputs with no external RNG crate.
    fn deterministic_sets() -> Vec<Vec<CandidateMetrics>> {
        let mut sets = vec![
            // Empty and single-element edge cases.
            Vec::new(),
            vec![metric(1.0, 3)],
            // Exact ties on both objectives (duplicates must all survive).
            vec![metric(1.0, 2), metric(1.0, 2), metric(1.0, 2)],
            // Ties on error, different complexity (only min complexity kept).
            vec![metric(1.0, 5), metric(1.0, 2), metric(1.0, 2), metric(1.0, 8)],
            // Ties on complexity, different error.
            vec![metric(3.0, 4), metric(1.0, 4), metric(2.0, 4)],
            // A clean staircase frontier plus interior dominated points.
            vec![metric(0.1, 9), metric(0.2, 5), metric(0.4, 2), metric(0.3, 7), metric(0.25, 6)],
        ];

        // Several pseudo-random sets of assorted sizes.
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (state >> 33) as u32
        };
        for size in [2usize, 3, 7, 16, 40, 128] {
            let candidates = (0..size)
                .map(|_| {
                    // Small integer-valued buckets guarantee frequent ties.
                    let error = (next() % 8) as f64;
                    let complexity = (next() % 8) as usize;
                    metric(error, complexity)
                })
                .collect();
            sets.push(candidates);
        }
        sets
    }

    #[test]
    fn pareto_front_2d_matches_the_quadratic_front_on_every_deterministic_set() {
        for candidates in deterministic_sets() {
            assert_eq!(
                pareto_front_2d(&candidates),
                pareto_front(&candidates),
                "fronts diverged for {candidates:?}"
            );
        }
    }

    #[test]
    fn pareto_front_2d_handles_empty_and_single() {
        assert_eq!(pareto_front_2d(&[]), Vec::<usize>::new());
        assert_eq!(pareto_front_2d(&[metric(2.0, 4)]), vec![0]);
    }

    #[test]
    fn pareto_front_2d_keeps_all_exact_duplicates() {
        let candidates = vec![metric(1.0, 2), metric(1.0, 2), metric(0.5, 3)];
        assert_eq!(pareto_front_2d(&candidates), vec![0, 1, 2]);
    }

    #[test]
    fn pareto_front_2d_is_bit_for_bit_deterministic() {
        let candidates = deterministic_sets().pop().unwrap();
        assert_eq!(pareto_front_2d(&candidates), pareto_front_2d(&candidates));
    }
}
