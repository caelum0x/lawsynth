# Interval coverage

`confidence_interval` takes empirical values and a validated probability level, then returns percentile endpoints. `percentile` sorts finite values and uses the package’s deterministic rank interpolation policy.

An interval’s coverage interpretation depends on the data-generating process, statistic, and resampling design. A percentile interval alone does not guarantee nominal frequentist coverage for dependent, biased, selected, or misspecified data.
