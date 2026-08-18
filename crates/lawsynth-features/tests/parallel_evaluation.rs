//! Bit-identity and edge-case contract tests for parallel feature evaluation.
//!
//! The headline guarantee: `evaluate_parallel(dataset, k)` equals the serial
//! `evaluate(dataset)` to the last bit for every thread count `k`, including
//! `k` larger than the row count and `k` that does not divide the rows evenly.

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_features::{FeatureLibrary, FeatureMatrix, row_partitions};

/// Thread counts exercised everywhere: 1 (serial fallback), small factors,
/// primes, a value that does not divide typical row counts, and a value that
/// will exceed small datasets so every chunk holds a single row.
const THREAD_COUNTS: [usize; 6] = [1, 2, 3, 4, 7, 8];

fn ident(name: &str) -> Identifier {
    Identifier::new(name).unwrap()
}

/// Builds a deterministic 3-variable dataset with `rows` samples. Values are
/// chosen to be non-integer and to swing sign so trig, rational, interaction,
/// and high-degree polynomial terms all produce distinctive bit patterns.
fn dataset(rows: usize) -> Dataset {
    let time = (0..rows).map(|i| i as f64).collect::<Vec<_>>();
    let x = (0..rows).map(|i| (i as f64) * 0.37 - 3.1).collect::<Vec<_>>();
    let y = (0..rows).map(|i| (i as f64).sin() * 2.0 + 0.5).collect::<Vec<_>>();
    let z = (0..rows).map(|i| ((i as f64) * 0.13).cos() - (i as f64) * 0.02).collect::<Vec<_>>();
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

/// A representative spread of libraries: polynomial degrees with and without the
/// intercept, and each optional family (trig, rational, interaction) mixed in.
fn libraries() -> Vec<FeatureLibrary> {
    let vars = [ident("x"), ident("y"), ident("z")];
    let mut result = Vec::new();

    for degree in 1..=3 {
        for include_constant in [true, false] {
            result
                .push(FeatureLibrary::polynomial(vars.clone(), degree, include_constant).unwrap());
        }
    }

    let mut poly_trig = FeatureLibrary::polynomial(vars.clone(), 2, true).unwrap();
    poly_trig.extend(FeatureLibrary::trigonometric(vars.clone()).unwrap());
    result.push(poly_trig);

    let mut poly_rational = FeatureLibrary::polynomial(vars.clone(), 2, false).unwrap();
    poly_rational.extend(FeatureLibrary::bounded_rational(vars.clone()).unwrap());
    result.push(poly_rational);

    let mut poly_interaction = FeatureLibrary::polynomial(vars.clone(), 1, true).unwrap();
    poly_interaction.extend(FeatureLibrary::interactions(vars.clone()).unwrap());
    result.push(poly_interaction);

    // Everything at once, degree 3.
    let mut combined = FeatureLibrary::polynomial(vars.clone(), 3, true).unwrap();
    combined.extend(FeatureLibrary::trigonometric(vars.clone()).unwrap());
    combined.extend(FeatureLibrary::bounded_rational(vars.clone()).unwrap());
    combined.extend(FeatureLibrary::interactions(vars).unwrap());
    result.push(combined);

    result
}

/// Asserts two matrices are identical down to the raw IEEE-754 bit pattern of
/// every entry, not merely `==` (which would treat `-0.0`/`0.0` as equal and
/// NaN as unequal). This is the strict bit-identity check.
fn assert_bit_identical(serial: &FeatureMatrix, parallel: &FeatureMatrix) {
    assert_eq!(serial.terms, parallel.terms, "term columns diverged");
    assert_eq!(serial.rows.len(), parallel.rows.len(), "row count diverged");
    for (row_index, (serial_row, parallel_row)) in
        serial.rows.iter().zip(&parallel.rows).enumerate()
    {
        assert_eq!(serial_row.len(), parallel_row.len(), "row {row_index} width diverged");
        for (col_index, (a, b)) in serial_row.iter().zip(parallel_row).enumerate() {
            assert_eq!(
                a.to_bits(),
                b.to_bits(),
                "bit mismatch at row {row_index}, column {col_index}: {a} vs {b}"
            );
        }
    }
}

#[test]
fn parallel_matches_serial_bit_for_bit_across_libraries_and_sizes() {
    // Row counts include primes and non-multiples of the thread counts so the
    // final chunk is uneven for most (rows, threads) combinations.
    let row_counts = [2, 3, 5, 17, 64, 101];
    for rows in row_counts {
        let data = dataset(rows);
        for library in libraries() {
            let serial = library.evaluate(&data).unwrap();
            for threads in THREAD_COUNTS {
                let parallel = library.evaluate_parallel(&data, threads).unwrap();
                assert_bit_identical(&serial, &parallel);
            }
        }
    }
}

#[test]
fn parallel_matches_serial_when_threads_exceed_rows() {
    let data = dataset(3);
    let library =
        FeatureLibrary::polynomial([ident("x"), ident("y"), ident("z")], 3, true).unwrap();
    let serial = library.evaluate(&data).unwrap();
    for threads in [4, 8, 64, 1_000] {
        let parallel = library.evaluate_parallel(&data, threads).unwrap();
        assert_bit_identical(&serial, &parallel);
    }
}

#[test]
fn result_is_invariant_to_thread_count() {
    // Determinism does not depend on the number of threads: every thread count
    // must yield the exact same matrix as thread count 2.
    let data = dataset(97);
    let library = {
        let vars = [ident("x"), ident("y"), ident("z")];
        let mut lib = FeatureLibrary::polynomial(vars.clone(), 3, true).unwrap();
        lib.extend(FeatureLibrary::trigonometric(vars).unwrap());
        lib
    };
    let baseline = library.evaluate_parallel(&data, 2).unwrap();
    for threads in [1, 3, 4, 5, 7, 8, 16, 32] {
        let candidate = library.evaluate_parallel(&data, threads).unwrap();
        assert_bit_identical(&baseline, &candidate);
    }
}

#[test]
fn zero_rows_are_handled_at_the_partition_boundary() {
    // A `Dataset` cannot hold 0 rows — `TimeAxis::new` rejects an empty axis — so
    // the 0-row case is reachable only inside the partition kernel. It must yield
    // no chunks (hence no worker threads and an empty matrix) for any thread
    // count, which is the invariant the parallel path relies on.
    assert!(TimeAxis::new(vec![]).is_err());
    for threads in [0, 1, 2, 8, 100] {
        assert!(row_partitions(0, threads).is_empty());
    }
}

#[test]
fn single_row_matches_serial_on_every_thread_count() {
    let data = dataset(1);
    let library =
        FeatureLibrary::polynomial([ident("x"), ident("y"), ident("z")], 2, true).unwrap();
    let serial = library.evaluate(&data).unwrap();
    assert_eq!(serial.rows.len(), 1);
    for threads in [0, 1, 2, 8, 100] {
        let parallel = library.evaluate_parallel(&data, threads).unwrap();
        assert_bit_identical(&serial, &parallel);
    }
}

#[test]
fn zero_threads_fall_back_to_the_serial_path() {
    let data = dataset(40);
    let library =
        FeatureLibrary::polynomial([ident("x"), ident("y"), ident("z")], 3, true).unwrap();
    let serial = library.evaluate(&data).unwrap();
    let parallel = library.evaluate_parallel(&data, 0).unwrap();
    assert_bit_identical(&serial, &parallel);
}

#[test]
fn parallel_path_uses_the_requested_deterministic_chunking() {
    // The public partition helper is the exact rule the parallel path splits by,
    // so verifying it here verifies the chunk boundaries the workers receive.
    assert_eq!(row_partitions(100, 4), vec![0..25, 25..50, 50..75, 75..100]);
    // Uneven: 100 rows over 7 threads -> first `100 % 7 == 2` chunks get +1.
    let partitions = row_partitions(100, 7);
    assert_eq!(partitions.len(), 7);
    assert_eq!(partitions.first().unwrap(), &(0..15));
    assert_eq!(partitions.last().unwrap().end, 100);
    let covered: usize = partitions.iter().map(|range| range.end - range.start).sum();
    assert_eq!(covered, 100);
    // Cap: more threads than rows collapses to one chunk per row.
    assert_eq!(row_partitions(5, 999).len(), 5);
}

#[test]
fn evaluation_values_are_correct_not_just_consistent() {
    // Guards against both paths sharing the same wrong kernel: check a known row.
    let x = ident("x");
    let y = ident("y");
    let data = Dataset::new(
        TimeAxis::new(vec![0.0, 1.0, 2.0, 3.0, 4.0]).unwrap(),
        [
            NumericColumn::new(x.clone(), vec![2.0, 3.0, 4.0, 5.0, 6.0]),
            NumericColumn::new(y.clone(), vec![5.0, 7.0, 9.0, 11.0, 13.0]),
        ],
    )
    .unwrap();
    let library = FeatureLibrary::polynomial([x, y], 2, true).unwrap();
    let matrix = library.evaluate_parallel(&data, 4).unwrap();
    // Column order matches the serial test in library.rs: [1, y, x, y^2, x*y, x^2].
    assert_eq!(matrix.rows[0], vec![1.0, 5.0, 2.0, 25.0, 10.0, 4.0]);
    assert_eq!(matrix.rows[4], vec![1.0, 13.0, 6.0, 169.0, 78.0, 36.0]);
}
