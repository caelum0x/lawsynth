# Propagation contract

`linear_propagate(gradient, covariance)` returns `sqrt(gᵀΣg)`. The gradient dimension must equal the checked covariance dimension and every gradient value must be finite. Materially negative computed variance (`< -1e-12`) returns `NonPositiveVariance`; negligible negative round-off is clamped to zero before the square root.

`monte_carlo_propagate(inputs, config, model)` samples each `Samples` input column independently with replacement for exactly `draws` evaluations. The resulting `Samples` has exactly that length. A zero draw count, no inputs, or a non-finite model result is an error.

Empirical propagation intentionally does not preserve cross-input dependence. A caller must use `linear_propagate` with a covariance matrix or implement a joint sampler when correlation matters.
