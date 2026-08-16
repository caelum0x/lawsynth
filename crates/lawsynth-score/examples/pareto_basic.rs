use lawsynth_score::{CandidateMetrics, pareto_front};

fn main() {
    let candidates = [
        CandidateMetrics {
            mean_squared_error: 0.1,
            complexity: 4,
        },
        CandidateMetrics {
            mean_squared_error: 0.2,
            complexity: 1,
        },
        CandidateMetrics {
            mean_squared_error: 0.3,
            complexity: 6,
        },
    ];
    println!(
        "Pareto-optimal candidate indexes: {:?}",
        pareto_front(&candidates)
    );
}
