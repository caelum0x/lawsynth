# Lagged observations and time order

`lagged_observations(series, lag)` emits observations for indices `lag..series.len()`. Each output has `target = series[i]` and `history = [series[i-1], …, series[i-lag]]`; history is most-recent-first. Lag zero, insufficient observations, and non-finite values are rejected.

`validate_time_order(times)` accepts only a non-empty strictly increasing finite sequence and returns its first time, last time, and count. The first non-finite or non-increasing position returns `NonMonotonicTime { index }`.

No resampling or irregular-time lag interpolation is implemented. Time validation checks order, not cadence or causal identifiability.
