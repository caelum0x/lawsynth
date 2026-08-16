# Execution reports

`lawsynth_runner::classify_result` converts the final result of one execution
into an `ExecutionReport`. Its statuses are `Succeeded`, `Failed`,
`Cancelled`, and `Rejected`. Cancellation maps only `RunnerError::Cancelled`;
capacity exhaustion and an invalid envelope map to `Rejected`; every other
runner error maps to `Failed`.

Reports contain the caller-supplied work id and an optional human-readable
message. They are snapshots, not transitions: no `queued`, `started`, retry,
worker identity, timestamp, result checksum, or persistence behavior exists.
The caller must retain the original envelope if it needs those facts.
