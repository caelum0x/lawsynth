# World lab workflow

A host application can implement a local scenario editor that validates state, parameter, and input identifiers against an inspected bundle, then calls the CLI or Python simulation boundary. Store each scenario as immutable data: initial values, overrides, scheduled changes, horizon, step, units, and source bundle hash.

Use `@lawsynth/state-store` for draft UI state only. Promote a run only after the execution boundary returns a valid trajectory or a typed error. Do not persist an optimistic chart as if simulation completed.

There is no shipped browser simulation engine, remote execution service, or collaborative world lab. Any such system needs explicit resource limits and security controls outside these packages.
