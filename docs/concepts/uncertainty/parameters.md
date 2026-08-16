# Parameter uncertainty

Use `Samples` and `bootstrap` to summarize a scalar parameter estimator, or use `CovarianceMatrix` with `linear_propagate` when you have a local gradient and an estimated joint covariance. `profile_quadratic` fits a local quadratic profile over supplied parameter-score points.

These tools do not fit a parameter posterior, compute a Hessian from a World, or infer parameter covariance from a discovery candidate. Supply the estimator and evaluation domain explicitly.
