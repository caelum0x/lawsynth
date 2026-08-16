# Regime schedules

`RegimeSchedule` holds sorted, mutually exclusive `RegimeInterval { regime, start, end }` values. Each interval is half-open `[start, end)`, and both bounds must be finite with `end > start`. Input is sorted by `start`, then `end`; any pair with `earlier.end > later.start` is rejected as overlapping. Adjacent intervals are allowed.

`active_at(t)` returns the interval satisfying `start <= t < end`, or none for a non-finite time or a gap. A schedule labels time only. Version 0.1 does not associate distinct equations with a regime, trigger switches during simulation, or serialize schedules in world bundles.
