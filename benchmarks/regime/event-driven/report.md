# regime/event-driven report

## Method

The generator creates 128 deterministic observations from a seed derived from
the benchmark identity, and writes the observed columns to a CSV. The compiled
`lawsynth discover` command runs with `--regimes` (plus `--pareto` and
`--refine`) and emits a regime_segments signal.

## Result

The benchmark passes when discovery and inspection both succeed and the emitted
regime_segments meets the ground-truth-derived minimum in `[expect]`. This is a real
partial execution signal for regime segmentation; it does not claim exact recovery of the
oracle structure.
