# LawSynth guide

A cohesive, end-to-end walkthrough of LawSynth: build the CLI, discover a
governing-equation world from a CSV, then simplify, analyze, control, export, and
quantify uncertainty — every command verified against the current build.

LawSynth is a deterministic, offline, Rust-first toolkit that discovers governing
equations from time-series data and packages them as executable `.lsworld`
worlds. Nothing in this guide touches the network, and every run is reproducible.

## Pages

| Page | What it covers |
| --- | --- |
| [getting-started.md](./getting-started.md) | Build `lawsynth-cli`, the core concept, and a first `discover → explain → simulate` run on a shipped dataset. |
| [workflow.md](./workflow.md) | The full pipeline: discover → simplify → stability → control → export → validate → uncertainty, each a real command with real output. |
| [determinism.md](./determinism.md) | Why re-running discovery yields a byte-identical world, shown with `shasum`/`cmp`, plus what is and isn't guaranteed. |

## Runnable examples

Everything shown in these pages is executable and CI-checkable:

| File | Purpose |
| --- | --- |
| [`examples/run_all.sh`](./examples/run_all.sh) | Runs every documented command against the built binary and asserts each exits 0. Locates the binary via `$LAWSYNTH_BIN` → `target/debug` → `target/release` → `cargo run`. |
| [`examples/lotka-volterra.csv`](./examples/lotka-volterra.csv) | Deterministic 200-sample predator–prey dataset (regenerate with `lawsynth new lotka-volterra ... --samples 200`). |
| [`examples/forced-oscillator.csv`](./examples/forced-oscillator.csv) | Deterministic forced (controlled) dataset for `lawsynth control`. |
| [`examples/gen_forced_oscillator.py`](./examples/gen_forced_oscillator.py) | Pure-`math`, RNG-free generator for the forced dataset (byte-reproducible). |

Run the whole guide as a test:

```sh
bash docs/guide/examples/run_all.sh
```

## Boundary specifications

This guide links each feature to its `specs/` contract for the honest limits and
semantics of that feature (it does not restate the math). Key contracts:

- [`discovery-run`](../../specs/discovery-run), [`template-priors`](../../specs/template-priors) — discovery and candidate-library contracts.
- [`simulation-contract`](../../specs/simulation-contract) — solver, time-grid, and trajectory semantics.
- [`egraph-simplification`](../../specs/egraph-simplification), [`structural-reductions`](../../specs/structural-reductions) — simplification.
- [`stability-analysis`](../../specs/stability-analysis), [`analytic-jacobian`](../../specs/analytic-jacobian) — fixed points and classification.
- [`controlled-discovery`](../../specs/controlled-discovery), [`control-design`](../../specs/control-design) — forced-system (SINDYc) discovery.
- [`uncertainty-contract`](../../specs/uncertainty-contract), [`coefficient-uncertainty`](../../specs/coefficient-uncertainty) — confidence bands and coefficient uncertainty.
- [`reproducibility`](../../specs/reproducibility) — determinism, hashing, and versioning.
- [`domain-packs`](../../specs/domain-packs) — curated, self-validating domain presets.

See [`specs/README.md`](../../specs/README.md) for the full contract index.

## Authoritative command list

`lawsynth help` prints the subcommands your build actually ships. If a capability
appears in `specs/` but not in `lawsynth help` (e.g. bifurcation analysis,
implicit/DAE dynamics), it is a library crate rather than a CLI command — this
guide documents only what the CLI exposes today.
