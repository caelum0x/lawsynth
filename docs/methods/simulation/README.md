# Simulation

`lawsynth-sim` evaluates typed continuous and discrete worlds deterministically. Continuous worlds use classical fourth-order Runge–Kutta on a fixed maximum step; discrete worlds evaluate all next-state laws against the same current context. Inputs, parameters, and scheduled changes are validated before integration.

The crate also offers a standalone diagonal-noise Euler–Maruyama integrator. Hybrid event segmentation is a scheduling primitive, not a complete event-handling solver.
