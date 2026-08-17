//! Timing harness for the P10 partitioned feature-library evaluation.
//!
//! Measures single-node vs partitioned feature-matrix wall-clock on a large
//! synthetic dataset (many samples x many candidate features) and reports the
//! speedup, then asserts the partitioned matrices are bit-identical. This
//! isolates the genuinely parallel step (feature evaluation) from the rest of
//! the pipeline, which stays single-node for exact float reductions.
//!
//! Run with: `cargo bench -p lawsynth-discovery --bench distributed_throughput`.

use std::{hint::black_box, time::Instant};

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_discovery::evaluate_library_partitioned;
use lawsynth_features::FeatureLibrary;

fn main() {
    let samples = 40_000usize;
    let names = ["a", "b", "c", "d", "e"];
    let ids = names.map(|name| Identifier::new(name).unwrap());

    let time = (0..samples).map(|step| step as f64 * 1e-3).collect::<Vec<_>>();
    let columns = ids
        .iter()
        .enumerate()
        .map(|(index, id)| {
            let phase = 0.3 + index as f64 * 0.11;
            let values =
                time.iter().map(|t| (phase * t).sin() + 0.2 * t - 0.05 * t * t).collect::<Vec<_>>();
            NumericColumn::new(id.clone(), values)
        })
        .collect::<Vec<_>>();
    let dataset = Dataset::new(TimeAxis::new(time).unwrap(), columns).unwrap();

    // Degree-3 polynomial + trig + rational over 5 variables => a wide library.
    let mut library = FeatureLibrary::polynomial(ids.clone(), 3, true).unwrap();
    library.extend(FeatureLibrary::trigonometric(ids.clone()).unwrap());
    library.extend(FeatureLibrary::bounded_rational(ids).unwrap());
    let feature_count = library.terms().len();

    println!(
        "dataset: {samples} samples x {} variables => {feature_count} candidate features",
        names.len()
    );

    // Warm up caches/branch predictors and capture the single-node baseline.
    let baseline = evaluate_library_partitioned(&library, &dataset, 1).unwrap();

    let repeats = 5;
    let mut single_node = f64::INFINITY;
    for partitions in [1usize, 2, 4, 8] {
        let mut best = f64::INFINITY;
        for _ in 0..repeats {
            let started = Instant::now();
            let matrix = evaluate_library_partitioned(&library, &dataset, partitions).unwrap();
            let elapsed = started.elapsed().as_secs_f64();
            black_box(&matrix);
            best = best.min(elapsed);

            // Correctness gate: every partition count is bit-identical.
            assert_eq!(matrix.rows.len(), baseline.rows.len());
            for (got, want) in matrix.rows.iter().zip(&baseline.rows) {
                for (lhs, rhs) in got.iter().zip(want) {
                    assert_eq!(lhs.to_bits(), rhs.to_bits(), "partition {partitions} diverged");
                }
            }
        }
        if partitions == 1 {
            single_node = best;
        }
        let speedup = single_node / best;
        println!(
            "partitions={partitions:>2}  best={:>9.3} ms  speedup={speedup:>5.2}x  (bit-identical)",
            best * 1e3
        );
    }
}
