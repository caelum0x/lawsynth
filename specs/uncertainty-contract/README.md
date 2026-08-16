# Uncertainty contract

`lawsynth-uncertainty` provides deterministic, in-memory numerical primitives. It validates data before calculation and returns `UncertaintyError` instead of dropping observations, imputing values, or choosing a distribution implicitly.

The implemented surface is `Samples`, covariance matrices, bootstrap and percentile intervals, first-order and empirical propagation, quadratic profiling, and explicitly declared structural sources. It is not a posterior-inference engine, a distribution-fitting API, or a trajectory uncertainty solver.

Every public numerical input must be finite unless a constructor documents a narrower condition. Seeds make bootstrap and Monte-Carlo output reproducible for the same inputs, configuration, and callback.
