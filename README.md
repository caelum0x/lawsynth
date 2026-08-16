# LawSynth

LawSynth is a local, deterministic toolkit for turning time-series observations
into executable mathematical worlds.  The Rust workspace validates a compact
World IR, evaluates scalar expressions, discovers sparse state laws from CSV
data, serializes worlds as `.lsworld` bundles, and simulates continuous or
discrete worlds.

## What is implemented

- validated identifiers, units, expressions, variables, parameters, and state
  transition laws;
- deterministic numerical simulation (RK4 for continuous worlds and a
  discrete stepping path), scheduled parameter/input interventions, and CSV
  trajectory output;
- CSV numeric data ingestion, profiling, finite-difference/smoothed
  derivatives, polynomial/trigonometric/rational feature libraries, sparse
  regression, scoring, and symbolic candidate rendering;
- canonical `.lsworld` bundle read/write with integrity checks;
- a `lawsynth` CLI and a Python package whose native module is built with
  maturin.

The repository deliberately does **not** present unimplemented service,
plugin, Studio, causal-inference, regime, or uncertainty packages as working
products.  Those directories describe later architectural scope; their
capability boundaries are explicit in code and tests.

## Quick start

Install the Rust toolchain pinned in `rust-toolchain.toml`, then run:

```sh
cargo test --workspace
cargo run -p lawsynth-cli -- --help
```

Discover a continuous world from a numeric CSV file:

```sh
cargo run -p lawsynth-cli -- discover observations.csv \
  --time time --state x,y --output recovered.lsworld
cargo run -p lawsynth-cli -- inspect recovered.lsworld
```

See the CLI usage text for supported discovery and simulation options.  The
checked-in examples and conformance cases are executable and are the source of
truth for supported inputs.

## Python package

The pure-Python data and configuration layer is under `python/lawsynth`; the
native extension is built from `crates/lawsynth-python`:

```sh
cd python/lawsynth
python -m pip install maturin
maturin develop
python -m pytest -q tests
```

## Development

The project keeps Cargo network access offline by default for reproducible
local checks.  Run `cargo fetch` in an environment that permits registry
access before enabling a clean machine build.  See
[CONTRIBUTING.md](CONTRIBUTING.md) for the verification matrix and
[ARCHITECTURE.md](ARCHITECTURE.md) for the implementation boundary.

## License

Licensed under [Apache-2.0](LICENSE).  Third-party notices, when required, are
recorded in [NOTICE](NOTICE).
