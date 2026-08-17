# lawsynth-schema-gen

`lawsynth-schema-gen` turns the LawSynth specification contracts into machine
consumable type definitions. The contracts (World IR bundle payload, variables,
parameters, laws, and observation datasets described under `specs/world-ir`) are
kept in one typed registry so the emitted JSON Schema, TypeScript, and Python
outputs cannot drift apart — they are all projections of the same source.

The tool is dependency-free, deterministic, and offline. It never fetches
schemas from a network location and produces byte-identical output for a given
contract set.

## Usage

```bash
# List available contracts
lawsynth-schema-gen list

# Emit JSON Schema (draft 2020-12) for every contract to stdout
lawsynth-schema-gen json

# Emit TypeScript interfaces for one contract
lawsynth-schema-gen ts --contract WorldBundle

# Write one JSON Schema file per contract into a directory
lawsynth-schema-gen json --out build/schemas

# Emit Python dataclasses
lawsynth-schema-gen py --out build/types
```

## Output

- `json` — one draft 2020-12 schema per contract, with `additionalProperties:
  false`, enum/pattern constraints, and `$ref` links between contracts.
- `ts` — `export interface` declarations with optional fields and string-literal
  unions for enumerations.
- `py` — frozen `@dataclass` definitions with `Literal[...]` enums and optional
  fields ordered last.

## Boundaries

The tool describes contracts; it does not validate live `.lsworld` bundles or
run discovery. Validation against these schemas is the responsibility of the
consuming service or notebook.
