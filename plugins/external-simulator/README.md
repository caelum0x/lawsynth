# external-simulator

A LawSynth **simulator plugin** that runs as an isolated child process. The host
spawns the plugin executable and exchanges validated payloads over the
length-delimited frame protocol defined by
[`lawsynth-plugin-api`](../../crates/lawsynth-plugin-api).

This crate provides the simulator *compute core*, `LinearSimulator`, which
integrates a linear ODE system `dx/dt = A·x + b` with an explicit-Euler scheme
and a bounded step size. The same core is used two ways:

- **In-process / trusted:** call `LinearSimulator::simulate` directly (this is
  what the example and tests do).
- **Out-of-process:** wrap the core in a small `main` that reads request frames
  from stdin and writes response frames to stdout — the mode the `plugin.toml`
  manifest advertises (`kind = "process"`).

## Why a separate process

Simulation is untrusted, CPU-heavy work. Process isolation lets the host apply
OS-level sandboxing, CPU/memory limits, and hard timeouts that the protocol
alone cannot guarantee. See
[specs/plugin-protocol/permissions.md](../../specs/plugin-protocol/permissions.md).

## Protocol handshake (honest description)

The transport is the `Frame` format from the plugin API: a 4-byte big-endian
body length, 2-byte protocol version (`1`), 1-byte kind, 1 reserved zero byte,
an 8-byte request id, then the payload. The host and plugin exchange:

1. `Hello` (kind `1`) — the host confirms protocol version 1 and negotiates the
   maximum frame size.
2. `Request` (kind `2`) — a serialized `SimulationRequest` (`initial_state` and
   strictly increasing `times`).
3. `Response` (kind `3`) — a serialized `SimulationResponse` (a dense
   `states[time][state]` matrix) **or** `Error` (kind `4`) carrying a
   `PluginError`.
4. `Shutdown` (kind `5`) — graceful drain and exit.

**Payload encoding is not fixed by the API** (the crate specifies framing, not
serialization); a conforming worker and host agree on one encoding out of band.
Every request and response is validated by `lawsynth-plugin-api` before it
crosses the boundary.

## Layout

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Standalone crate with a path dependency on the plugin API. |
| `src/lib.rs` | `LinearSimulator` implementing `SimulationPlugin`. |
| `plugin.toml` | Manifest advertising `kind = "process"`. |
| `examples/basic.rs` | Runs the simulator core on a decay ODE. |
| `tests/plugin_test.rs` | Contract tests for the simulator. |
| `docs/usage.md` | Request/response contract and worker wiring. |

## Quick start

```bash
cargo test --manifest-path plugins/external-simulator/Cargo.toml
cargo run  --manifest-path plugins/external-simulator/Cargo.toml --example basic
```

## License

Apache-2.0. See [LICENSE](LICENSE).
