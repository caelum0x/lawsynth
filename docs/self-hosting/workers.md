# Workers and background execution

No worker, job queue, scheduler, or distributed run executor is implemented. The local server accepts and stores run metadata and validates an embedded simulation specification, but it does not execute a discovery or simulation job in the background.

For a trusted single-host workflow, invoke the Rust CLI or Python SDK from an external supervisor, write outputs to a dedicated directory, validate them with `lawsynth inspect`, then create the corresponding server records and artifacts through the application boundary. The supervisor must enforce resource limits, cancellation, retry policy, and process isolation.

Do not claim asynchronous execution based on a `runs` route alone. Production queue integrations require an explicit worker protocol, durable status transitions, authorization propagation, and artifact-commit rules.
