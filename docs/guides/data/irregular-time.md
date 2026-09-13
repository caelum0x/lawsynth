---
title: Irregular sampling in time-series data
description: Learn how LawSynth handles irregular sampling, preserves real timestamps, chooses derivative estimators, and treats gaps without silent resampling.
tags: irregular sampling, time-series data, non-uniform timestamps, resampling
---

# Irregular sampling in time-series data

**Irregular sampling** means the elapsed time between observations is not constant. LawSynth accepts a non-uniform time axis when every timestamp is finite and the sequence is strictly increasing. Keep the actual timestamps in the CSV or `Dataset`; replacing them with row numbers changes the physical time scale unless every interval is genuinely equal.

## Before discovery

Inspect the sequence of time differences, not only the average interval. Very small gaps can amplify measurement noise in a derivative estimate, while a large unobserved gap can make a local derivative misleading. Duplicate, non-finite, or out-of-order timestamps must be corrected upstream because the engine does not silently sort or merge them.

Choose the estimator to match the data:

- the [three-point finite-difference method](../../methods/differentiation/finite.md) supports valid non-uniform grids and remains a local estimate;
- the [natural cubic-spline method](../../methods/differentiation/spline.md) also accepts non-uniform times and uses the full series;
- spectral differentiation requires a regular grid and deliberately rejects irregular input.

Compare reasonable derivative choices and discovery results on the same chronological holdout interval. Agreement is useful evidence; it is not proof that sampling gaps are harmless.

## Resampling and gaps

LawSynth does not invent a regular grid or choose an interpolation policy. If you resample upstream, record the interpolation method, target grid, and excluded regions so the transformation can be reproduced. Check that discovered terms remain stable under reasonable changes to that choice.

For discontinuous measurements or a long interval with no observations, segment the experiment instead of forcing one estimator across the gap. Regime-aware and missing-observation models are not part of the current discovery API.

For the implemented validation contract and estimator boundaries, see [differentiation on irregular time grids](../../methods/differentiation/irregular.md).
