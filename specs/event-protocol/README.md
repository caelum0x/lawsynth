# Event protocol boundary

LawSynth 0.1 has no network event protocol, broker, durable event store, or
subscription API. The event-shaped values implemented today are local Rust
values: `lawsynth_core::ProgressEvent`, `lawsynth_runner::ExecutionReport`,
and `lawsynth_runner::Heartbeat`. This specification defines the semantics a
caller can safely rely on when it receives those values from the process that
created them.

No document in this directory defines JSON, Protobuf, WebSocket, retry, or
delivery semantics. A service that exposes these values must define its own
wire schema and authentication boundary; it must not describe that transport
as supplied by the current engine.
