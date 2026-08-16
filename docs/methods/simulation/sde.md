# Stochastic differential equations

`euler_maruyama` is a standalone vector integrator for diagonal-noise SDEs `dx = drift dt + diffusion dW`. It uses a reproducible seeded generator, Box–Muller standard-normal draws, a positive maximum step, and a shortened final step.

Drift and diffusion closures must return one finite value per state. This implementation has no correlated noise, Milstein correction, adaptive stepping, weak/strong error control, or automatic conversion from a `World` to an SDE.
