# `@lawsynth/world-schema`

Dependency-free TypeScript contracts and validation for the LawSynth world boundary.

This package does not decode `.lsworld` archives or simulate a world. Those are
Rust responsibilities (`lawsynth-bundle` and `lawsynth-sim`). Its job is to
reject source data that cannot be represented by the current Rust core before a
client submits it for encoding.

## Current executable boundary

The bundle manifest is exactly:

```json
{
  "format": "lawsynth-world",
  "format_version": "0.1.0",
  "world_encoding": "lawsynth-world-binary-v1"
}
```

`validateManifest` accepts only those three fields. This mirrors the Rust
reader, which compares the manifest bytes against its current supported form.

`validateRustWorldSource` validates the JSON-shaped source form for the scalar
Rust World IR:

- variables have one of the implemented roles: `state`, `control`,
  `exogenous`, `observed`, `latent`, or `derived`;
- parameters and expression constants are finite scalar numbers;
- only continuous or discrete laws are accepted, with exactly one law per
  state variable;
- expressions use constants, symbols, `neg|exp|log|sin|cos`, and
  `add|sub|mul|div|pow`, with Rust's depth limit of 127 nested nodes;
- identifier, symbol, unit, and dimensional checks mirror the current Rust
  `Identifier`, `Unit`, and World construction rules.

Events, regimes, stochastic laws, custom calls, delay expressions, graph
metadata, signatures, and catalog metadata are deliberately rejected by that
validator because the current binary world encoding cannot persist them. The
related TypeScript interfaces are descriptive data contracts for future-facing
tools; they must not be treated as executable-core support.

## Usage

```ts
import { assertRustWorldSource, validateManifest } from "@lawsynth/world-schema";

const manifest = validateManifest({
  format: "lawsynth-world",
  format_version: "0.1.0",
  world_encoding: "lawsynth-world-binary-v1",
});

const world = assertRustWorldSource({
  formatVersion: "0.1.0",
  id: "decay",
  time: { kind: "continuous", unit: "s" },
  variables: [{ id: "x", role: "state", unit: "1" }],
  parameters: [{ id: "rate", value: 0.2, unit: "1/s" }],
  laws: [{
    kind: "continuous",
    target: "x",
    expression: {
      kind: "binary", operator: "mul",
      left: { kind: "unary", operator: "neg", operand: { kind: "symbol", id: "rate" } },
      right: { kind: "symbol", id: "x" },
    },
  }],
});
```

`validateBundleCatalog` is intentionally separate from `validateManifest`: a
catalog describes artifacts for a UI or registry, while `manifest.json` is the
small fixed document embedded in a Rust bundle.

## Development

Run `npm test` inside this package (or `tsc -p tsconfig.json --noEmit` for a
type-only check). The test runner uses no test framework or runtime dependency.
