# api-doc-gen

Generate LawSynth API documentation across every language surface.

The Rust crate `crates/lawsynth-api-types` is the authoritative definition of
LawSynth's public API values. This tool scans that crate once and renders
documentation for each surface so the docs never drift from the types.

## Surfaces

- **openapi** — an OpenAPI 3.1 JSON document. Component schemas are derived
  directly from the Rust types (enums become string enums, structs become object
  schemas with correct `required` sets, identifiers become strings). Per
  `specs/service-api/resources.md` no endpoint paths are normative in this
  release, so the generated read paths are illustrative and labelled as such.
- **rust** — Markdown reference for the Rust types.
- **python** — Markdown reference for the Python surface (`lawsynth`).
- **typescript** — Markdown reference for the TypeScript surface.
- **all** — writes every surface into an output directory.

Output is deterministic (types sorted, JSON keys sorted), so docs can be
committed and diffed.

## Usage

```sh
# Print the OpenAPI document
python src/main.py --surface openapi

# Write every surface into ./api-docs
python src/main.py --surface all --out api-docs

# Write a single Markdown file
python src/main.py --surface rust --out api-docs/rust.md
```

Installed as a package it exposes the `api-doc-gen` console script.

## Development

```sh
python -m pytest tools/api-doc-gen/tests
```

Pure standard library, deterministic, and offline.
