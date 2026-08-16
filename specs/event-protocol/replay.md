# Replay

There is no event log to replay. Progress trackers and heartbeats are process
memory only, while execution reports are returned directly to the caller.
Restarting a process resets tracker sequences and loses heartbeat state.

Scientific replay is instead based on preserving inputs, the `EngineConfig`
seed and version, the selected algorithms, and produced bundle bytes. Exact
replay can still differ if a caller changes data, configuration, dependency
build, hardware, or numerical execution path; this repository does not yet
provide a replay executor or verifier.
