# Summary semantics

The implemented summaries are arithmetic mean, unbiased sample variance, standard error, empirical percentile, bootstrap percentile interval, covariance, and quadratic profile summaries. They operate on supplied values only; no missing-value policy exists because non-finite values are rejected at entry.

Covariance estimated from rows divides by `rows.len() - 1`. Each row must have the same non-zero dimension, all values must be finite, and at least two rows are required. The result is row-major and symmetric by construction.

No skewness, kurtosis, robust estimators, weighted estimates, Bayesian credible intervals, or multiple-comparison correction is implemented by this crate.
