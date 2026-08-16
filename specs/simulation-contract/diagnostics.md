# Diagnostics and failures

Simulation failures are returned as typed SimulationError values. Validation
covers time grids, required/unknown initial state identifiers, unknown parameter
overrides or inputs, invalid state input targets, non-finite input values,
invalid scheduled-change times, expression evaluation errors, and non-finite
computed values.

The simulator does not return partial trajectories, warning collections, local
truncation errors, or tolerance estimates. A failed call has no successful
trajectory result; callers that need telemetry may use tracing around the
in-process invocation.
