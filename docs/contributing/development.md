# Development workflow

Use the pinned Rust 1.94.0 toolchain and Python 3.11+. The root Makefile and
justfile expose the baseline verification matrix:

```sh
make fmt
make check
make test
make lint
make python-test
# or: make verify
```

`make verify` formats, checks, tests, lints, and runs Python SDK tests. It may
rewrite formatting, so use `cargo fmt --all -- --check` when a read-only CI
check is needed. Run `cargo test -p CRATE` while iterating, then the workspace
suite before review.

Cargo is offline by default. New dependencies require a deliberate lockfile
update and a reproducible way to populate the cache; never hide a fetch behind
tests. The local pre-commit hooks run Rust format checking and Python syntax
compilation.

For changes under `python/lawsynth`, run
`PYTHONPATH=python/lawsynth/src python3 -m pytest -q python/lawsynth/tests`.
Use `maturin develop` before tests that need the native extension.

## TypeScript packages

Node 22 and pnpm 10.18.2 are the workspace baseline. `pnpm-workspace.yaml`
enrolls only `world-schema`, `api-client`, `state-store`, `chart-core`, and
`design-system`; do not run workspace checks as evidence that planned apps or
unenrolled packages work. After installing the locked workspace dependencies,
run:

```sh
pnpm build
pnpm typecheck
pnpm test
```

For a focused package, change into `packages/world-schema` (or another
enrolled package) and run its declared `npm test` / `npm run typecheck` script.
The TypeScript layer validates source contracts and UI data; it does not
replace Rust bundle decoding or simulation.
