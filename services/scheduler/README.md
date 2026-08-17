# LawSynth scheduler service

`lawsynth-scheduler` is a local, durable scheduler for the worker's typed
`JobEnvelope` values. It owns queue state, worker-pool placement, generation-fenced
leases, expiry recovery, cancellation, and dead-letter transitions, and persists
lifecycle checkpoints through `lawsynth-store`. It is a library first
(`Scheduler<S>`) with an optional HTTP/1.1 control-plane transport
(`SchedulerServer`). No async runtime, HTTP framework, or message broker is linked.

## Boundaries

- **Control-plane only over HTTP.** The transport exposes exactly the scheduler's
  serializable operations: health, pool registration, job state, checkpoints,
  cancellation, expiry recovery. Lease/heartbeat/complete/fail carry or fence
  executable `JobEnvelope` values, which have no wire codec, so they stay
  in-process Rust API calls. `SchedulerTransport` declares this honestly:
  `LocalTyped` is real executable dispatch; `HttpControlPlane` is the serializable
  subset. See `docs/security.md`.
- **Deterministic in caller time.** Placement, lease expiry, and recovery take
  `now_ms` explicitly, so routing is a pure function of `(scheduler, now_ms,
  request)` and is testable without a socket or a clock.
- **Durable, bounded checkpoints.** Records are persisted through an `ObjectStore`
  and bounded by `maximum_checkpoint_bytes`; the payload is never serialized.

## CLI

```
lawsynth-scheduler serve <addr>          # control plane over an in-memory store
lawsynth-scheduler serve <addr> <root>   # control plane over a durable LocalStore
```

Example:

```sh
lawsynth-scheduler serve 127.0.0.1:8082
lawsynth-scheduler serve 0.0.0.0:8082 /var/lib/lawsynth/scheduler
```

Run with no subcommand and it prints the honest transport statement (executable
dispatch is in-process and typed; the HTTP surface is control-plane only) and
exits. `serve` prints one startup line to stderr and then blocks, handling one
request per connection on a thread-per-connection model.

## HTTP surface

See `docs/api.md`. In brief: `GET /health`, `POST /pools`, `GET /jobs/{id}`,
`GET /jobs/{id}/checkpoint`, `POST /jobs/{id}/cancel`, and `POST /recover`.

## Configuration

Bounds and per-environment profiles live under `config/` (`limits.yaml`,
`development.yaml`, `test.yaml`, `staging.yaml`, `production.yaml`,
`logging.yaml`). They document the fields of `SchedulerConfig` (`src/config.rs`)
and `StoreConfig`; the current CLI applies the built-in defaults reproduced in
`limits.yaml`. No environment variables are read — see `.env.example`.

## Build and test

```sh
cargo build --release -p lawsynth-scheduler
cargo test -p lawsynth-scheduler
```

Further reading: `docs/architecture.md`, `docs/api.md`, `docs/operations.md`,
`docs/failures.md`, `docs/security.md`.
