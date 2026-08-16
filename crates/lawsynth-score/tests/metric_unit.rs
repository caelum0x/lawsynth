use lawsynth_score::{CandidateMetrics, ScoringConfig, rank_candidates, weighted_rank};

#[test]
fn domination_and_weighted_ranking_use_the_documented_minimization_order() {
    let simple = CandidateMetrics {
        mean_squared_error: 1.0,
        complexity: 1,
    };
    let worse = CandidateMetrics {
        mean_squared_error: 2.0,
        complexity: 3,
    };
    let accurate = CandidateMetrics {
        mean_squared_error: 0.5,
        complexity: 10,
    };
    assert!(simple.dominates(worse));
    assert!(!accurate.dominates(simple));
    assert_eq!(rank_candidates(&[simple, worse, accurate]), vec![2, 0, 1]);
    assert_eq!(
        weighted_rank(
            &[simple, worse, accurate],
            ScoringConfig {
                error_weight: 1.0,
                complexity_weight: 0.2
            },
        )
        .unwrap()
        .into_iter()
        .map(|(index, _)| index)
        .collect::<Vec<_>>(),
        vec![0, 2, 1]
    );
}
