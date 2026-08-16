# Trajectory propagation boundary

First-order propagation is available as `sqrt(gᵀΣg)` through `linear_propagate`. It returns an error only for materially negative computed variance and otherwise clamps small negative round-off to zero.

There is no trajectory-level covariance integration, sensitivity-equation solver, ensemble world simulator, or uncertainty band construction in this crate.
