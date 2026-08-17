# conformance-runner

Discover and run LawSynth cross-language conformance cases, then report pass/fail
deterministically.

LawSynth guarantees that Rust, Python, and TypeScript surfaces agree on the
`.lsworld` wire format and world semantics. The checked-in cases under
`tests/conformance/` and `tests/cross-language/` encode that contract: each is a
directory with a `case.toml` descriptor, an `input.json` fixture, an
`expected.json` observable outcome, and an executable `run.py` that builds real
bundles and drives the native CLI.

## What it does

- **Discover** — walks a root directory for `case.toml` and loads each case
  (sorted by id for reproducible output).
- **Execute** — runs each case's declared runner in its own directory and
  captures the trailing JSON result it prints.
- **Compare** — checks the observed result against `expected.json` with
  float-tolerant, recursive comparison that reports every mismatch.
- **Report** — prints a text or JSON summary and exits non-zero if any case fails.

The comparison and reporting layers are pure functions with an injectable runner,
so the tool is unit-testable without a Rust toolchain.

## Usage

```sh
# Run every conformance case
python src/main.py run tests/conformance

# Filter by id and emit JSON
python src/main.py run tests/cross-language --filter roundtrip --json
```

Installed as a package it exposes the `conformance-runner` console script.

## Development

```sh
python -m pytest tools/conformance-runner/tests
```
