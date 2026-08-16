# Contributing

Contributions must describe a real, testable behavior. Do not add placeholder
APIs, fabricated benchmark outcomes, mocks that replace the component under
test, or documentation that advertises unavailable capabilities.

## Local checks

Use the pinned Rust toolchain and run:

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

For Python package changes:

```sh
PYTHONPATH=python/lawsynth/src python -m pytest -q python/lawsynth/tests
```

Run affected conformance, scientific, and example workflows directly. Tests
should exercise the public crate, CLI, or built native module whenever one
exists; a capability that has not been implemented should fail explicitly and
be documented as such.

## Change discipline

- Keep public formats canonical and backward-compatible unless a versioned
  migration accompanies the change.
- Add a regression test for defects and a reproducible fixture for numerical
  claims.
- State numerical assumptions, tolerances, and determinism properties in the
  relevant module documentation.
- Keep dependencies minimal; update lockfiles with dependency changes.
- Do not commit secrets, generated build outputs, or local datasets.

Open a pull request with a concise problem statement, implementation notes,
verification commands, and any known capability boundary.
