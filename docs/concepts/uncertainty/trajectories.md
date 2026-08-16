# Trajectory uncertainty

Generate each trajectory by applying a chosen parameter or input draw to a `SimulationRequest`, then simulate and aggregate aligned values in application code. Store the solver step and each draw definition because discretization affects the output distribution.

The simulation crate returns one deterministic trajectory per request. It does not generate confidence bands, estimate trajectory covariance, or propagate a World’s parameter uncertainty automatically.
