# causal/nonlinear report

## Method

The generator creates 128 deterministic observations from a seed derived from
the benchmark identity, and writes the observed columns to a CSV. The compiled
`lawsynth discover` command runs with `--causal` (plus `--pareto` and
`--refine`) and emits a dependency_edges signal.

## Result

The benchmark passes when discovery and inspection both succeed and the emitted
dependency_edges meets the ground-truth-derived minimum in `[expect]`. This is a real
partial execution signal for dependency-hypothesis edges; it does not claim exact recovery of the
oracle structure.
