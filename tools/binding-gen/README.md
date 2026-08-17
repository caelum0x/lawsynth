# binding-gen

Generate aligned client bindings from the LawSynth public API type surface.

The Rust crate `crates/lawsynth-api-types` is the single source of truth for the
transport-neutral values that cross LawSynth's HTTP, CLI, and job APIs
(`ProjectId`, `RunSummary`, `SimulationRequest`, and friends). This tool scans
that crate and re-emits the same shapes as stubs for other languages so no
binding drifts from the Rust definitions.

## What it does

- Scans `crates/lawsynth-api-types/src/*.rs` for public enums, newtype
  identifiers, and structs into a small language-neutral schema (IR).
- Emits, from that IR:
  - **python** — frozen dataclasses, `Enum`s, and `NewType` aliases
  - **typescript** — `interface`s, string-literal union enums, and type aliases
  - **proto** — a proto3 schema (messages, enums, `repeated`/`optional` fields)
  - **rust** — a prelude that re-exports the discovered surface

Output is deterministic (types are sorted), so generated files can be committed
and diffed.

## Usage

```sh
# Print TypeScript declarations
python src/main.py --lang typescript

# Write a proto file from an explicit crate path
python src/main.py --lang proto --crate crates/lawsynth-api-types --out api.proto
```

Installed as a package it exposes the `binding-gen` console script.

## Development

```sh
python -m pytest tools/binding-gen/tests
```

Pure standard library, deterministic, and offline.
