# Ordering

Ordering is local only. A `ProgressTracker` assigns sequence `0` to its first
accepted report and increments by one for each subsequent accepted report.
Within a stage, a later fraction must be greater than or equal to the previous
fraction. Equal fractions are valid, allowing a caller to update a message
without advancing work.

There is no global sequence, logical clock, timestamp, total ordering between
trackers, or ordering guarantee for `ExecutionReport` and `Heartbeat`. An
adapter that merges events from workers must carry a run/attempt identity and
define merge ordering itself.
