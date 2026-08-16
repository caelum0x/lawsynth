# LawSynth worker

`lawsynth-worker` is a synchronous, local execution boundary for typed LawSynth jobs.
It accepts validated `JobEnvelope` values and executes the engine's real discovery
pipeline or deterministic RK4 simulator. It is a library first; the executable only
states the supported surface and deliberately does not open a listener.

Each accepted job reserves its declared CPU, memory, and disk budget through
`lawsynth-runner`, then writes a durable lifecycle record to an `ObjectStore` before
and after execution. The local `LocalStore` implementation makes those records
survive a worker restart. Job results remain typed in memory rather than being
silently converted to an incomplete queue codec.

The only supported transport is `TransportSurface::LocalDirect`. Queue, HTTP/RPC,
plugins, uploads, OS sandboxing, and distributed leases are not linked and are
reported as unavailable rather than represented by placeholders.

Run the focused verification with `cargo test -p lawsynth-worker`.
