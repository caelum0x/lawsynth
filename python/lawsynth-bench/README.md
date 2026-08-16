# lawsynth-bench

`lawsynth-bench` is a dependency-free analysis package for recorded LawSynth benchmark results. It does not run a solver and it never manufactures timing, recovery, or accuracy values. Instead it validates JSON observations, aggregates repeated samples, compares compatible result sets, detects thresholded regressions, and renders reproducible reports.

```bash
lawsynth-bench summarize fixtures/metrics/sample.json
lawsynth-bench compare baseline.json candidate.json --format json
```

Each observation has `problem`, `implementation`, `metric`, `value`, and optional `unit`, `run_id`, and string labels. Measurements can be produced by a CI job or an external harness; this package preserves their provenance rather than claiming it created them.
