# Architecture

The scheduler is a synchronous, local core with an optional HTTP control-plane
transport layered explicitly on top. The network surface is a deliberate,
narrow addition, never an ambient capability.

## Core (`Scheduler<S>`)

`Scheduler::new(SchedulerConfig, store)` owns the whole lifecycle:

- queue state and admission bounded by `maximum_queued_jobs`;
- worker-pool placement over registered `WorkerPool` capacities;
- lease issue and fencing (`lease.rs`) with generation-based staleness;
- expiry recovery (`recover_expired`) driven by the caller-supplied `now_ms`;
- cancellation and dead-letter transitions after `maximum_attempts`;
- durable lifecycle checkpoints persisted through an `ObjectStore`
  (`PersistedCheckpoint`), bounded by `maximum_checkpoint_bytes`.

Time is always an argument (`now_ms`), so placement, expiry, and cancellation are
deterministic and testable without a clock. It links no broker, RPC listener, or
payload codec.

## Transport (`SchedulerServer`)

The optional `http` module implements a blocking, thread-per-connection HTTP/1.1
server on `std::net` with no framework. It parses one bounded request, locks the
shared `Arc<Mutex<Scheduler>>`, routes through `router::route` (a pure function),
and writes the response. `with_system_clock` injects the wall clock for `main`.
Serializing control-plane mutations behind the mutex preserves the core's
deterministic contract. A poisoned lock is recovered rather than propagated, so
one panicking connection cannot wedge the control plane.

## The no-codec boundary

`JobEnvelope` is a typed, in-memory value. Encoding it for the wire would require
a codec the crate deliberately does not implement, so the operations that carry
or fence an envelope — lease, heartbeat, complete, fail — are **not** exposed over
HTTP. `SchedulerTransport` names this honestly: `LocalTyped` is the real
executable dispatch surface; `HttpControlPlane` is the serializable subset. An
embedder that needs remote dispatch must supply its own authenticated transport
and a complete envelope codec.

## Persistence

Checkpoints are published atomically by the `LocalStore` when `serve` is given a
durable root, so a lifecycle record survives a restart. With no root, `serve`
runs against an in-memory store and checkpoints are process-lifetime only.
