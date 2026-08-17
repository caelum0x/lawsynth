//! Distributed-discovery path (P10): partitioned feature-library evaluation.
//!
//! This module implements the "Distributed discovery" clause of
//! `specs/hosted-platform/README.md`: discovery MAY be partitioned across
//! workers *provided the result is identical to the single-node result for the
//! same inputs and config*. The partitioned path here is additive and opt-in —
//! the default single-node pipeline is untouched and byte-identical.
//!
//! # What is genuinely parallelized
//!
//! The embarrassingly-parallel step is **feature-library evaluation**: building
//! the candidate design matrix `rows[r][c] = evaluate(term[c], env[r])`. Every
//! cell is a pure function of a single term and a single row's environment, with
//! no cross-term or cross-row reduction. That makes column partitioning
//! *order-independent*: a cell computes to the same `f64` bits whether it is
//! evaluated inside the single-node double loop or alone on a worker thread.
//!
//! # What stays single-node for exactness (honest note)
//!
//! The sparse-regression stage sums per-state residual-sum-of-squares into a
//! running `total_rss`. Floating-point addition is not associative, so that
//! reduction is order-dependent and is deliberately left on the single-node
//! sequential path to preserve bit-identical results. Only the order-independent
//! work (feature evaluation) is partitioned. See requirement 4 of the P10 task.
//!
//! # How bit-identical determinism is guaranteed
//!
//! 1. Columns are split into `P` deterministic contiguous partitions by
//!    [`partition_boundaries`] (a pure function of term count and `P`).
//! 2. Each partition evaluates its column block with the *same* per-row
//!    environments and the *same* `evaluate` call as the single-node path.
//! 3. Partitions are reassembled in canonical partition-index order, so the
//!    final column order is exactly `[0, N)` regardless of thread completion
//!    timing. Determinism comes from canonical ordering, never from scheduling.
//!
//! When `partitions <= 1` the code calls [`FeatureLibrary::evaluate`] directly,
//! so the default discovery path is literally the original computation.

use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::thread;

use lawsynth_data::Dataset;
use lawsynth_expr::{Environment, evaluate};
use lawsynth_features::{FeatureError, FeatureLibrary, FeatureMatrix, FeatureTerm};

use crate::{
    CancellationToken, DiscoveryCheckpoint, DiscoveryConfig, DiscoveryError, DiscoveryResult,
};

/// Runs the full discovery pipeline with the feature-library evaluation split
/// across `partitions` deterministic column partitions using real in-process
/// threads.
///
/// The returned [`DiscoveryResult`] is **identical** to
/// [`discover`](crate::discover) for the same `dataset` and `config`, for any
/// `partitions >= 1` — this is the load-bearing P10 guarantee. `partitions` is
/// clamped to at least one; higher counts than the number of candidate features
/// simply leave later partitions empty.
pub fn discover_partitioned(
    dataset: &Dataset,
    config: &DiscoveryConfig,
    partitions: usize,
) -> Result<DiscoveryResult, DiscoveryError> {
    let partitions = NonZeroUsize::new(partitions.max(1)).expect("partitions is at least one");
    let mut checkpoint = DiscoveryCheckpoint::new(dataset.fingerprint());
    crate::execute::run_discovery(
        dataset,
        config,
        &CancellationToken::default(),
        &mut checkpoint,
        partitions,
    )
}

/// Evaluates a feature library with `partitions` deterministic column
/// partitions, returning a matrix bit-identical to
/// [`FeatureLibrary::evaluate`].
///
/// Exposed publicly so timing harnesses (and callers that want just the
/// partitioned matrix) can isolate the genuinely parallel step without running
/// the whole pipeline.
pub fn evaluate_library_partitioned(
    library: &FeatureLibrary,
    dataset: &Dataset,
    partitions: usize,
) -> Result<FeatureMatrix, FeatureError> {
    let partitions = NonZeroUsize::new(partitions.max(1)).expect("partitions is at least one");
    evaluate_library(library, dataset, partitions)
}

/// Internal entry point shared by the pipeline: single-node when `partitions`
/// is one, partitioned otherwise. The single-node branch calls the original
/// [`FeatureLibrary::evaluate`], keeping the default path byte-identical.
pub(crate) fn evaluate_library(
    library: &FeatureLibrary,
    dataset: &Dataset,
    partitions: NonZeroUsize,
) -> Result<FeatureMatrix, FeatureError> {
    let terms = library.terms();
    if partitions.get() <= 1 || terms.len() <= 1 {
        return library.evaluate(dataset);
    }

    // Per-row environments are shared, immutable, and built exactly as the
    // single-node loop builds them, so every worker sees identical inputs.
    let environments = build_environments(dataset);
    let boundaries = partition_boundaries(terms.len(), partitions.get());

    let partial: Vec<Vec<Vec<f64>>> = thread::scope(|scope| {
        let handles = boundaries
            .windows(2)
            .map(|window| {
                let (start, end) = (window[0], window[1]);
                let columns = &terms[start..end];
                let environments = &environments;
                scope.spawn(move || evaluate_columns(columns, environments))
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| handle.join().expect("feature partition thread panicked"))
            .collect::<Result<Vec<_>, _>>()
    })?;

    let rows = reassemble(partial, environments.len(), terms.len());
    Ok(FeatureMatrix { terms: terms.to_vec(), rows })
}

/// Builds one immutable [`Environment`] per sample row, mirroring the ordering
/// and contents of the single-node evaluation loop.
fn build_environments(dataset: &Dataset) -> Vec<Environment> {
    let columns = dataset.columns();
    (0..dataset.time().len())
        .map(|row| {
            columns
                .iter()
                .map(|(id, column)| (id.clone(), column.values[row]))
                .collect::<BTreeMap<_, _>>()
        })
        .collect()
}

/// Evaluates one contiguous column block for every row, producing a row-major
/// `[rows][block_width]` sub-matrix. Identical per-cell computation to the
/// single-node inner loop.
fn evaluate_columns(
    columns: &[FeatureTerm],
    environments: &[Environment],
) -> Result<Vec<Vec<f64>>, FeatureError> {
    environments
        .iter()
        .map(|environment| {
            columns
                .iter()
                .map(|term| {
                    evaluate(&term.expression, environment)
                        .map_err(|error| FeatureError::Evaluation(error.to_string()))
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .collect()
}

/// Concatenates partition column blocks back into canonical column order.
///
/// Partition `k` holds contiguous columns, and blocks arrive in partition-index
/// order, so appending each partition's row slice reproduces the single-node
/// column order `[0, N)` exactly.
fn reassemble(partial: Vec<Vec<Vec<f64>>>, num_rows: usize, num_terms: usize) -> Vec<Vec<f64>> {
    let mut rows = vec![Vec::with_capacity(num_terms); num_rows];
    for block in partial {
        for (row_index, columns) in block.into_iter().enumerate() {
            rows[row_index].extend(columns);
        }
    }
    rows
}

/// Splits `total` columns into at most `partitions` contiguous, balanced blocks
/// and returns the `blocks + 1` boundary offsets `[0, .., total]`.
///
/// The split is a deterministic pure function of its inputs: the first
/// `total % blocks` partitions take one extra column so widths differ by at most
/// one. `partitions` is capped at `total` to avoid empty blocks.
fn partition_boundaries(total: usize, partitions: usize) -> Vec<usize> {
    let blocks = partitions.min(total).max(1);
    let base = total / blocks;
    let remainder = total % blocks;
    let mut boundaries = Vec::with_capacity(blocks + 1);
    boundaries.push(0);
    let mut cursor = 0;
    for index in 0..blocks {
        cursor += base + usize::from(index < remainder);
        boundaries.push(cursor);
    }
    boundaries
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;
    use lawsynth_data::{NumericColumn, TimeAxis};

    use super::*;

    fn multi_variable_dataset() -> Dataset {
        let a = Identifier::new("a").unwrap();
        let b = Identifier::new("b").unwrap();
        let c = Identifier::new("c").unwrap();
        let time = (0..200).map(|step| step as f64 * 0.05).collect::<Vec<_>>();
        Dataset::new(
            TimeAxis::new(time.clone()).unwrap(),
            [
                NumericColumn::new(a, time.iter().map(|t| (0.7 * t).sin() + 0.3 * t).collect()),
                NumericColumn::new(b, time.iter().map(|t| (0.4 * t).cos() * t).collect()),
                NumericColumn::new(c, time.iter().map(|t| 0.2 * t * t - t).collect()),
            ],
        )
        .unwrap()
    }

    fn wide_library() -> FeatureLibrary {
        let ids = ["a", "b", "c"].map(|name| Identifier::new(name).unwrap());
        let mut library = FeatureLibrary::polynomial(ids.clone(), 3, true).unwrap();
        library.extend(FeatureLibrary::trigonometric(ids.clone()).unwrap());
        library.extend(FeatureLibrary::bounded_rational(ids).unwrap());
        library
    }

    #[test]
    fn partitioned_evaluation_is_bit_identical_to_single_node() {
        let dataset = multi_variable_dataset();
        let library = wide_library();
        let baseline = library.evaluate(&dataset).unwrap();
        for partitions in [1usize, 2, 3, 7, 64] {
            let partitioned = evaluate_library_partitioned(&library, &dataset, partitions).unwrap();
            assert_eq!(partitioned.terms, baseline.terms, "term order differs at p={partitions}");
            assert_eq!(
                partitioned.rows.len(),
                baseline.rows.len(),
                "row count differs at p={partitions}"
            );
            for (row_index, (got, want)) in partitioned.rows.iter().zip(&baseline.rows).enumerate()
            {
                assert_eq!(
                    got.len(),
                    want.len(),
                    "width differs at row {row_index}, p={partitions}"
                );
                for (col, (lhs, rhs)) in got.iter().zip(want).enumerate() {
                    assert_eq!(
                        lhs.to_bits(),
                        rhs.to_bits(),
                        "cell ({row_index},{col}) differs at p={partitions}"
                    );
                }
            }
        }
    }

    #[test]
    fn partition_boundaries_are_balanced_and_cover_all_columns() {
        assert_eq!(partition_boundaries(10, 3), vec![0, 4, 7, 10]);
        assert_eq!(partition_boundaries(7, 7), vec![0, 1, 2, 3, 4, 5, 6, 7]);
        // More partitions than columns collapse to one column per block.
        assert_eq!(partition_boundaries(3, 7), vec![0, 1, 2, 3]);
        // Every split covers [0, total] with no gaps or overlaps.
        for (total, partitions) in [(1, 1), (5, 2), (100, 7), (33, 33)] {
            let boundaries = partition_boundaries(total, partitions);
            assert_eq!(*boundaries.first().unwrap(), 0);
            assert_eq!(*boundaries.last().unwrap(), total);
            assert!(boundaries.windows(2).all(|window| window[0] <= window[1]));
        }
    }
}
