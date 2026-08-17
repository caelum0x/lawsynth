# custom-operator-rust

A reference LawSynth **algorithm plugin** written in Rust. It implements a
custom discovery operator against the stable
[`lawsynth-plugin-api`](../../crates/lawsynth-plugin-api) contract.

The operator, `LinearOperator`, performs a single-feature ordinary least
squares fit: given a tabular dataset and a numeric target column, it regresses
the target against every other numeric column and returns the best-fitting
linear law as an equation string plus a score (negative mean squared error).

## Boundaries

- **What it is:** an object-safe `AlgorithmPlugin` implementation plus a
  validated manifest and example. It is deliberately small and dependency-free
  so it can be compiled to a `wasm32-wasi` component and loaded by
  `lawsynth-plugin-host`.
- **What it is not:** a general symbolic regression engine. It only fits
  first-order linear laws and never mutates its input.
- **Trust model:** the plugin declares the `algorithm` and `dataset.read`
  capabilities. Declaration is not a grant — a host must include these in its
  granted policy before it executes the operator. See
  [specs/plugin-protocol/capabilities.md](../../specs/plugin-protocol/capabilities.md).

## Layout

| Path | Purpose |
| --- | --- |
| `Cargo.toml` | Standalone crate with a path dependency on the plugin API. |
| `src/lib.rs` | `LinearOperator` implementing `AlgorithmPlugin`. |
| `plugin.toml` | Manifest in the strict `key = value` grammar the host parses. |
| `examples/basic.rs` | Runnable end-to-end discovery example. |
| `tests/plugin_test.rs` | Integration tests for the operator contract. |
| `docs/usage.md` | Detailed usage, data-shape, and integration notes. |

## Quick start

```bash
cargo test --manifest-path plugins/custom-operator-rust/Cargo.toml
cargo run  --manifest-path plugins/custom-operator-rust/Cargo.toml --example basic
```

See [docs/usage.md](docs/usage.md) for the request/response contract.

## License

Apache-2.0. See [LICENSE](LICENSE).
