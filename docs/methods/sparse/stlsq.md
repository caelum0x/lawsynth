---
title: "STLSQ: Sequentially Thresholded Least Squares"
description: STLSQ means sequentially thresholded least squares, a sparse regression method used in SINDy. See its algorithm, threshold behavior, and runnable example.
tags: STLSQ, SINDy, sparse regression, equation discovery
---

# STLSQ: sequentially thresholded least squares

**STLSQ (sequentially thresholded least squares)** is a sparse regression method commonly used in SINDy-style equation discovery. In LawSynth, it fits candidate equation terms, removes coefficients below a chosen magnitude threshold, and refits only the terms that remain.

## How STLSQ works

1. Solve ridge-regularized least squares using the active feature columns.
2. Remove coefficients whose absolute value is smaller than `threshold`.
3. Refit the remaining columns.
4. Stop when the active set no longer changes or `max_iterations` is reached.

The result contains full-width coefficients, the active feature indices, and residual sum of squares. If the threshold removes every term, LawSynth returns an all-zero model rather than retaining stale coefficients.

## Run STLSQ in LawSynth

```bash
lawsynth discover observations.csv --time t --state x,y --output model.lsworld --solver stlsq --threshold 0.05
```

The threshold controls sparsity: a smaller value generally keeps more candidate terms, while a larger value produces a simpler equation but can remove weak dynamics. It is not a probability, significance level, or universal physical constant. Compare candidate thresholds using held-out trajectory error and equation complexity instead of choosing from derivative-space training error alone.

## Library defaults

- `threshold`: `0.05`
- `max_iterations`: `20`
- `ridge`: `1e-10`

These are the Rust library's `SparseConfig` defaults. CLI presets and explicit flags can override discovery settings.

## Standardized STLSQ

The Rust API also provides `stlsq_standardized`. It deterministically scales feature columns by their root-mean-square magnitude before fitting, then converts the coefficients back to the original feature scale. This helps when candidate terms have very different numeric magnitudes without changing the units of the returned model.

## Numerical behavior and limits

LawSynth solves the linear systems with deterministic Gaussian elimination and pivoting. Singular or malformed designs return errors; the solver does not silently substitute a pseudoinverse, cross-validation, or automatic threshold tuning.

Sparse term selection is model selection, not proof that excluded terms have no physical or causal effect. Validate the discovered equation on held-out trajectories and examine whether selected terms remain stable under reasonable changes to the threshold and data sample.

## Related guides

- [Choosing a sparse solver and threshold](../../guides/discovery/sparse.md)
- [`lawsynth discover` CLI reference](../../reference/cli/discover.md)
