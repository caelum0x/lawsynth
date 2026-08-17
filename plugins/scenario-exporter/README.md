# scenario-exporter

A LawSynth **scenario exporter plugin**. It serializes a discovered scenario —
a world plus the laws found for it — into a portable, deterministic artifact
that another tool can archive, diff, or re-import.

Built against the stable
[`lawsynth-plugin-api`](../../crates/lawsynth-plugin-api) and dependency-free, so
it compiles to a small `wasm32-wasi` artifact-writer that
`lawsynth-plugin-host` can sandbox.

## Supported formats

| Format | Media type | Notes |
| --- | --- | --- |
| `ExportFormat::Json` | `application/json` | Canonical, deterministically ordered JSON with full-precision numbers and correct string escaping. |
| `ExportFormat::World` | `text/plain; charset=utf-8` | The `var … / d(…)/dt = …` grammar consumed by `world-validator-wasi`, enabling a clean round-trip. |

## Guarantees

- **Deterministic:** serialization is a pure function of the scenario — the same
  input always yields byte-identical output.
- **Validated first:** a scenario is structurally validated before any bytes are
  written, so the exporter never emits an artifact that would fail to re-import
  (matching arities, unique valid variable names, finite state, non-empty laws,
  a bounded id).
- **Immutable:** the input scenario is never mutated.

## Capabilities

`plugin.toml` declares `artifact.write` (it produces an artifact) and
`dataset.read` (it reads a scenario). Declaration is not a grant — the host must
include these in its granted policy
(see [specs/plugin-protocol/capabilities.md](../../specs/plugin-protocol/capabilities.md)).

## Quick start

```bash
cargo test --manifest-path plugins/scenario-exporter/Cargo.toml
cargo run  --manifest-path plugins/scenario-exporter/Cargo.toml --example basic
```

See [docs/usage.md](docs/usage.md) for the API and artifact formats.

## License

Apache-2.0. See [LICENSE](LICENSE).
