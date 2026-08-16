# Empirical intervals

`percentile(values, p)` sorts a copy using total floating-point ordering and linearly interpolates ranks `p * (n - 1)`. It accepts only finite values and a finite `p` in `[0, 1]`; invalid probabilities use `InvalidConfidence`, empty data uses `EmptyInput`.

`confidence_interval` is a central *percentile bootstrap* interval. For confidence `c`, it returns percentiles `(1 - c)/2` and `1 - (1 - c)/2`. `IntervalConfig` requires finite `0 < c < 1`; a result with fewer than two estimates fails with `InsufficientResamples`.

The API does not promise coverage calibration, BCa correction, studentization, simultaneous intervals, or a confidence interval for a parameter without a caller-provided bootstrap statistic.
