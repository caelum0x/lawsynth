//! Serial-vs-parallel wall-clock comparison for feature-library evaluation.
//!
//! Matches the crate's `harness = false` bench style: a plain `main` that times
//! a fixed workload with `std::time::Instant` and prints throughput. It builds a
//! large synthetic dataset and a degree-3 combined library, then evaluates it
//! serially and with 1/2/4/8 threads. Each parallel run is asserted bit-identical
//! to the serial baseline so the benchmark can never report a speedup for a
//! result that quietly diverged.

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_features::{FeatureLibrary, FeatureMatrix};
use std::{hint::black_box, time::Instant};

const ROWS: usize = 60_000;
const REPEATS: usize = 5;

fn ident(name: &str) -> Identifier {
    Identifier::new(name).unwrap()
}

fn build_dataset() -> Dataset {
    let time = (0..ROWS).map(|i| i as f64).collect::<Vec<_>>();
    let x = (0..ROWS).map(|i| (i as f64) * 0.37 - 3.1).collect::<Vec<_>>();
    let y = (0..ROWS).map(|i| (i as f64).sin() * 2.0 + 0.5).collect::<Vec<_>>();
    let z = (0..ROWS).map(|i| ((i as f64) * 0.13).cos() - (i as f64) * 0.02).collect::<Vec<_>>();
    Dataset::new(
        TimeAxis::new(time).unwrap(),
        [
            NumericColumn::new(ident("x"), x),
            NumericColumn::new(ident("y"), y),
            NumericColumn::new(ident("z"), z),
        ],
    )
    .unwrap()
}

fn build_library() -> FeatureLibrary {
    let vars = [ident("x"), ident("y"), ident("z")];
    let mut library = FeatureLibrary::polynomial(vars.clone(), 3, true).unwrap();
    library.extend(FeatureLibrary::trigonometric(vars.clone()).unwrap());
    library.extend(FeatureLibrary::bounded_rational(vars).unwrap());
    library
}

fn matrices_bit_identical(a: &FeatureMatrix, b: &FeatureMatrix) -> bool {
    a.rows.len() == b.rows.len()
        && a.rows.iter().zip(&b.rows).all(|(ra, rb)| {
            ra.len() == rb.len() && ra.iter().zip(rb).all(|(va, vb)| va.to_bits() == vb.to_bits())
        })
}

/// Times `REPEATS` evaluations and returns the best (minimum) elapsed nanoseconds
/// to reduce noise from scheduler jitter.
fn best_nanos(mut run: impl FnMut() -> FeatureMatrix) -> u128 {
    let mut best = u128::MAX;
    for _ in 0..REPEATS {
        let started = Instant::now();
        let matrix = run();
        let elapsed = started.elapsed().as_nanos();
        black_box(&matrix);
        best = best.min(elapsed);
    }
    best
}

fn main() {
    let data = build_dataset();
    let library = build_library();
    let columns = library.terms().len();
    let cells = (ROWS * columns) as f64;

    println!(
        "parallel_eval: {ROWS} rows x {columns} candidate terms ({} available cores)",
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0)
    );

    let serial_matrix = library.evaluate(&data).unwrap();
    let serial_nanos = best_nanos(|| library.evaluate(&data).unwrap());
    report("serial", serial_nanos, serial_nanos, cells);

    for threads in [1usize, 2, 4, 8] {
        let matrix = library.evaluate_parallel(&data, threads).unwrap();
        assert!(
            matrices_bit_identical(&serial_matrix, &matrix),
            "parallel result with {threads} threads diverged from serial"
        );
        let nanos = best_nanos(|| library.evaluate_parallel(&data, threads).unwrap());
        report(&format!("parallel/{threads}"), nanos, serial_nanos, cells);
    }
}

fn report(label: &str, nanos: u128, serial_nanos: u128, cells: f64) {
    let seconds = nanos as f64 / 1e9;
    let throughput = cells / seconds / 1e6;
    let speedup = serial_nanos as f64 / nanos as f64;
    println!(
        "  {label:<12} {:>8.3} ms   {throughput:>8.1} Mcell/s   {speedup:>5.2}x vs serial",
        seconds * 1e3
    );
}
