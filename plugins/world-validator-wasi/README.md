# world-validator-wasi

A LawSynth **world validator plugin** compiled to WASI. It checks that a world —
a set of state variables, their initial values, and their time-derivative
expressions — is *structurally* well-formed before the host lets it enter a run.

The plugin mirrors the invariants enforced by the in-tree
[`lawsynth_wasm::World`](../../crates/lawsynth-wasm) type but is dependency-free
(only [`lawsynth-plugin-api`](../../crates/lawsynth-plugin-api)) so it produces a
small `wasm32-wasi` component that `lawsynth-plugin-host` can sandbox.

## What it checks

- Non-empty and matching arities: one initial value and one derivative per
  variable.
- Variable names are valid identifiers (`[A-Za-z_][A-Za-z0-9_]*`), unique, and
  never the reserved time symbol `t`.
- The initial state is finite.
- Derivative bodies are non-empty, NUL-free, and within a size bound.
- **Warnings** (non-fatal) for a derivative that references neither a state
  variable nor time.

Numeric evaluation of the expressions is intentionally *out of scope* — that is
the host's expression engine's job. This plugin guarantees a world is well-formed
enough to evaluate.

## Isolation and capability

`plugin.toml` declares `kind = "wasi"` and the single `world.validate`
capability. Declaration is not a grant: a host must include `world.validate` in
its granted policy before invoking the validator
(see [specs/plugin-protocol/capabilities.md](../../specs/plugin-protocol/capabilities.md)).

## WASI entrypoint

The module exports:

```rust
pub unsafe extern "C" fn lawsynth_world_validate(ptr: *const u8, len: usize) -> i32;
```

The host writes a UTF-8 world description into linear memory and calls the
export with a `(pointer, length)` pair. Return codes: `0` valid, `-1`
structurally invalid, `-2` not valid UTF-8, `-3` exceeds a validator resource
bound. Embedders that link the crate directly can call the richer, safe
`WorldValidator::validate_text` API instead.

## World description grammar

```text
# comments start with '#'
var <name> = <initial_value>
d(<name>)/dt = <expression text>
```

Lines are order-independent; every variable needs exactly one `var` line and one
derivative line.

## Quick start

```bash
cargo test --manifest-path plugins/world-validator-wasi/Cargo.toml
cargo run  --manifest-path plugins/world-validator-wasi/Cargo.toml --example basic
# Build the WASI artifact:
cargo build --manifest-path plugins/world-validator-wasi/Cargo.toml \
  --target wasm32-wasi --release
```

## License

Apache-2.0. See [LICENSE](LICENSE).
