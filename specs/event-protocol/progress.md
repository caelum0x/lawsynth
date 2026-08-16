# Progress events

`ProgressTracker::report` returns a `ProgressEvent` with a tracker-local,
zero-based sequence, one `ProgressStage`, a finite fraction in `[0, 1]`, and a
caller-provided message. Supported stages are `Input`, `Profiling`,
`Preprocessing`, `Differentiating`, `Features`, `Fitting`, `Scoring`, and
`Finalizing`.

Fractions cannot decrease for the same stage. Stage order is intentionally not
enforced: alternative execution paths may report stages in different orders.
Sequences increase for every accepted event from one tracker, but have no
meaning across trackers, runs, processes, retries, or restarts. The message is
not sanitized, size-bounded, persisted, or treated as telemetry.
