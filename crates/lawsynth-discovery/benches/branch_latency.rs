use lawsynth_discovery::DiscoveryBranch;
use lawsynth_score::CandidateMetrics;
use std::{hint::black_box, time::Instant};
fn main() {
    let branch = DiscoveryBranch::new(
        "sparse",
        "feature regression",
        CandidateMetrics { mean_squared_error: 0.1, complexity: 4 },
    );
    let started = Instant::now();
    let mut names = 0;
    for _ in 0..1_000_000 {
        names += black_box(branch.clone()).name.len();
    }
    println!("cloned {names} branch-name bytes in {:?}", started.elapsed());
}
