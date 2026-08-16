# Irregular sampling

The irregular-grid entry point validates that time is strictly increasing and finite, that signal values are finite, and that at least two aligned samples exist. It then delegates to the same three-point Lagrange estimator used by finite differences.

Irregular spacing is supported for this local estimator and the cubic-spline estimator. Spectral differentiation deliberately rejects irregular grids. No resampling, imputation, timestamp sorting, or duplicate-time merging is performed.
