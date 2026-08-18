# Deterministic parallel feature-library evaluation (v1)

This directory specifies the opt-in, multi-threaded evaluation path for
feature-library materialization implemented in `crates/lawsynth-features`
(`FeatureLibrary::evaluate_parallel`). It is a **boundary specification** in the
house style: it states what a conforming implementation MUST do.

## Motivation

Building the candidate matrix Θ(X) from a dataset is the hot inner loop of
discovery. Each row of Θ is the candidate terms evaluated at one sample, and rows
are independent of one another. The serial `FeatureLibrary::evaluate` walks rows
in order on a single thread; on large datasets and rich libraries this is
embarrassingly parallel work left on the table.

The parallel path spreads that per-row work across OS threads. Its non-negotiable
requirement is **bit-identity**: the parallel matrix MUST equal the serial matrix
to the last bit, for every thread count. Discovery downstream (sparse regression,
symbolic search) is deterministic and content-addressed; a parallel path that
changed a single low bit would break reproducibility. Speed is a bonus,
determinism is the contract.

## Requirements

1. **Parallelism is ONLY over independent rows.** A conforming parallel evaluation
   MUST split the row range `0..rows` into contiguous chunks and evaluate each
   chunk on its own worker. It MUST NOT introduce any cross-row reduction,
   atomics-based accumulation, or float summation whose order depends on
   threading. The per-row kernel — environment construction followed by
   term-by-term evaluation — MUST be the exact same code, in the exact same
   operation order, as the serial path. In the reference implementation the
   serial and parallel paths call one shared `evaluate_rows` kernel precisely so
   they cannot drift apart.

2. **Assembly is ordered.** Chunk outputs MUST be concatenated in ascending row
   order, independent of the order in which workers finish. Because each row runs
   identical float operations regardless of which thread computes it, and the
   rows are reassembled in their original order, the concatenated matrix is
   bit-identical to serial. This is the whole correctness argument: *identical
   per-row ops + ordered assembly ⇒ identical bits*.

3. **Deterministic chunk boundaries.** The split MUST depend only on `rows` and
   the requested thread count, never on scheduling. The reference rule
   (`row_partitions`): the chunk count is `clamp(threads, 1, rows)`, the base
   chunk size is `rows / chunks`, and the first `rows % chunks` chunks each take
   one extra row. The ranges tile `0..rows` exactly — contiguous, ordered, no
   gaps, no overlaps, no empty chunks.

4. **Serial fallback with no threads spawned.** `threads == 0` or `threads == 1`,
   a dataset of 0 or 1 rows, or any partition that degenerates to a single chunk
   MUST run the serial path directly, spawning no threads. `threads` MUST be
   capped at the row count so no worker receives an empty range.

5. **Result is invariant to thread count.** The number of threads MUST affect
   speed ONLY, never the result. Identical inputs MUST yield bit-identical output
   for every thread count `k` — including `k` larger than the row count and `k`
   that does not divide the rows evenly (an uneven final chunk). This is the
   headline test: `evaluate_parallel(.., k)` compared to `evaluate(..)` via
   `f64::to_bits` on every entry, for several libraries and dataset sizes.

6. **Offline and std-only.** The path MUST be deterministic and offline
   (`net.offline = true`). It MUST use only the standard library
   (`std::thread::scope`, `std::sync`) — NO external crates (no rayon). Scoped
   threads borrow the dataset and library directly; no `Arc` cloning of data is
   required.

7. **Honest limits.** Any speedup claim MUST be backed by a reproducible
   measurement (the `parallel_eval` bench times serial vs 1/2/4/8 threads on a
   large synthetic dataset and asserts each parallel result is bit-identical to
   serial before reporting). Speedup is bounded by available cores and, for
   cheap per-term work, by memory bandwidth. Small inputs do NOT benefit: thread
   spawn and join overhead dominates below a crossover row count, and the serial
   fallback covers the trivial cases. No parallel float reduction is used —
   precisely to preserve bit-identity — so there is no reduction-tree speedup to
   claim, only the linear-ish row fan-out bounded by core count.

## Non-goals

Per-term (intra-row) parallelism, a global thread pool, SIMD vectorization, and
work-stealing schedulers are out of scope for this boundary. They may be added as
extensions with their own contracts, provided they preserve the bit-identity
guarantee above.
