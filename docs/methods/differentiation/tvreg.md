# Total-variation regularization

`tvreg_smoothed_series` solves the one-dimensional ROF objective `0.5 ||x-y||² + lambda ||D x||₁` with deterministic ADMM iterations, a fixed `rho = 1`, and a residual tolerance of `1e-9`. `tvreg_series` differentiates the resulting signal with the finite-difference estimator.

`lambda` must be positive and finite and the iteration count nonzero. The penalty is on adjacent index differences, not elapsed time, so uneven sampling changes its physical interpretation. No convergence status, adaptive penalty, weighted observations, or higher-order TV model is exposed.
