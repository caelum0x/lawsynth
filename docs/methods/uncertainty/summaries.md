# Sample and interval summaries

`Samples` validates finite nonempty scalar observations and provides mean, unbiased sample variance, and standard error. `percentile` sorts values and uses linearly interpolated empirical quantiles. `confidence_interval` uses central bootstrap percentiles and requires at least two estimates.

These are descriptive finite-sample calculations. They do not correct bias, use BCa acceleration, or provide simultaneous intervals.
