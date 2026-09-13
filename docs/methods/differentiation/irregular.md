---
title: Differentiation on irregular time grids
description: See which LawSynth derivative estimators accept non-uniform timestamps, what the validator rejects, and why spectral differentiation needs a regular grid.
tags: irregular grid differentiation, finite differences, cubic spline, derivative estimation
---

# Differentiation on irregular time grids

LawSynth differentiates a valid non-uniform time series without first resampling it. The irregular-grid entry point validates the time and signal arrays, then delegates to the same three-point Lagrange estimator used by finite differences.

## Input contract

The estimator requires:

- at least two aligned time and signal values;
- finite timestamps and finite signal values;
- a strictly increasing time axis, with no duplicate timestamps.

It does not sort timestamps, merge duplicates, impute missing values, or select an interpolation rule. A validation failure is explicit rather than a silent data transformation.

## Supported estimators

The local [finite-difference estimator](finite.md) uses the adjacent left and right spacings for each interior point. Its three-point formula is exact for a quadratic evaluated on a valid non-uniform grid, but—as with other numerical derivatives—it can amplify measurement noise.

The [natural cubic-spline estimator](spline.md) also accepts strictly increasing non-uniform times and returns a derivative at each knot. Because it uses a global spline system, its behavior differs from the local three-point estimate.

Spectral differentiation deliberately rejects irregular grids. Resampling before a spectral method is a separate modeling decision and can introduce interpolation artifacts.

For data preparation, gap handling, and validation advice, see the [irregular sampling guide](../../guides/data/irregular-time.md).
