# Trajectory boundary

This crate has no trajectory type and performs no temporal integration. Its propagation functions accept a caller callback and return scalar samples only; any trajectory-shaped output must be produced and validated by the simulation layer.

Consequently, no contract exists here for temporal correlation, state covariance recursion, stochastic differential equations, observation noise alignment, or confidence bands along a simulation timeline.

For a time-indexed uncertainty study, callers must explicitly define a joint sampling or state-space procedure and may use these scalar primitives only where their assumptions hold.
