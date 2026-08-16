# Propagation

`linear_propagate(gradient, covariance)` computes the first-order variance `gᵀΣg` after validating dimensions. Use it when a differentiable output has a credible local linear approximation around the uncertainty region.

`monte_carlo_propagate` samples one empirical column per input dimension under a configured fixed seed and evaluates a caller closure. It treats those input columns as independent.

Neither path derives a gradient through the World IR or preserves joint empirical dependence in the Monte Carlo sampler.
