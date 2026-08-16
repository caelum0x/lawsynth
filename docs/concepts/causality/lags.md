# Lags

`lagged_observations(series, lag)` pairs each response with the observation `lag` positions earlier. It rejects zero lag and series too short to form a pair. This produces transparent scalar inputs for predictive tests.

Lag indices count observations, not elapsed physical time. Resample or account for irregular intervals before interpreting a lag as a duration.

The helper does not choose lag order, impute missing samples, or align separate timestamp axes.
