# Installation

The core requires Rust **1.94.0** with `clippy` and `rustfmt`; the pinned
toolchain is recorded in `rust-toolchain.toml`. The Python package requires
Python 3.11 or newer and uses maturin to build `lawsynth._native`.

Install the `lawsynth` CLI, or run it in place while developing:

```sh
cargo install --path crates/lawsynth-cli   # installs the `lawsynth` binary
# or
cargo run -p lawsynth-cli -- --help
cargo test --workspace
```

Cargo is deliberately configured with `[net] offline = true`. A fresh machine
must first obtain the lockfile's crates in an environment with registry access
(for example, temporarily run `cargo fetch` with offline mode disabled), then
restore offline mode for normal local verification. Do not edit the lockfile
or replace locked dependencies merely to make an offline build pass.

For Python development:

```sh
cd python/lawsynth
python -m pip install maturin
maturin develop
python -m pytest -q tests
```

This gives you the `lawsynth.Study` API. Two optional companion packages extend it:
`lawsynth-notebook` (the `StudyDashboard` rich notebook view) and
`lawsynth-connectors` (import observations from `filesystem`, `http`, `s3`,
`postgres`, and `sqlite` sources via `Study.from_source`). Both live under
`python/` and degrade to a clear error when absent.
