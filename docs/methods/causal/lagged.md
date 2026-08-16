# Lagged observations

`lagged_observations(series, lag)` constructs rows whose target is the current value and whose history is `[x(t-1), …, x(t-lag)]`. It requires a positive lag, more samples than the lag, and finite values.

The operation indexes sequence positions, not physical elapsed time. On irregular timestamps it does not interpolate or produce duration-aware lags.
