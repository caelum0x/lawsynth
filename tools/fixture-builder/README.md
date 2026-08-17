# lawsynth-fixture-builder

`lawsynth-fixture-builder` turns a declarative spec into deterministic,
canonically-encoded JSON test fixtures. It is used to materialise the sample
datasets and World IR bundle fixtures consumed by the apps and services (for
example `apps/*/fixtures/*.json`).

The tool is dependency-free and offline. Every fixture is produced from a fixed,
spec-derived seed and serialised with canonical JSON (sorted keys, two-space
indent, LF newlines, stable float formatting), so rebuilding a fixture yields
byte-identical files and SHA-256 checksums on any platform.

## Spec format

A spec file is a JSON list of fixtures. Each fixture has a `name`, a `type`, and
type-specific fields:

```json
[
  {
    "name": "decay",
    "type": "observation",
    "kind": "exponential_decay",
    "samples": 81,
    "step": 0.05,
    "parameters": { "rate": 1.0 }
  },
  {
    "name": "linear_world",
    "type": "world_bundle",
    "kind": "continuous",
    "variables": [{ "id": "x", "role": "State" }],
    "parameters": [{ "id": "k", "value": 1.0 }],
    "laws": [{ "target": "x", "expression": "-k * x" }]
  }
]
```

Observation kinds: `exponential_decay`, `harmonic`, `logistic_map`. Add
`"noise": <scale>` for seeded Gaussian noise that stays reproducible.

## Usage

```bash
# Build fixtures and a checksum manifest into a directory
lawsynth-fixture-builder build spec.json --out apps/studio/fixtures

# Print the manifest without writing files
lawsynth-fixture-builder build spec.json

# Verify on-disk fixtures still match a freshly built set
lawsynth-fixture-builder verify spec.json apps/studio/fixtures

# SHA-256 of any file
lawsynth-fixture-builder checksum path/to/file.json
```

## Boundaries

The builder produces fixture *content*; it does not run discovery or simulation.
World IR bundle fixtures reproduce the validated bundle shape (lexical ordering
of variables, parameters, and laws) but are not a substitute for engine
validation.
