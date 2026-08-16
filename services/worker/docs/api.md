# API contract

Construct `Worker<S>` from a validated `WorkerConfig` and an `ObjectStore`, create a
`JobEnvelope`, then call `execute` or deterministic `execute_at`. `Job::Discover`
calls the sparse discovery engine on a validated `Dataset`; `Job::Simulate` calls the
RK4 simulator for a validated `World` and scenario.

`checkpoint(job_id)` returns the last durable `JobCheckpoint`. Reusing an ID is
rejected so a caller cannot overwrite lifecycle evidence. `TransportSurface` is the
capability declaration: only `LocalDirect` is available.
