# Regime-indexed laws

`RegimeLawBook` maps a non-negative integer regime label to an `AffineLaw { intercept, slope }`. Insertion accepts only finite coefficients and overwrites the previous law for the same label. Its ordered map gives deterministic retrieval.

`evaluate(regime, x)` requires finite `x` and a registered regime; otherwise it returns `NonFiniteObservation { index: 0 }` or `InvalidParameter("unknown regime")`. Evaluation is exactly `intercept + slope * x`.

The book does not learn laws from segment data, resolve regime labels from observations, enforce continuity at boundaries, or support arbitrary symbolic equations.
