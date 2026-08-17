# lawsynth-license-check

`lawsynth-license-check` scans dependency manifests, verifies every dependency's
license against an allowlist, and can emit an attribution `NOTICE`. The default
allowlist mirrors the `[licenses].allow` set in the repository's `deny.toml`, so
the Python tooling stays in agreement with the Rust `cargo-deny` gate. Point it
at that file with `--policy deny.toml` to track any change automatically.

The tool is dependency-free and offline. It only reads the manifests it is
handed and never resolves packages over the network.

## Supported manifests

- `Cargo.lock` — package names and versions (Cargo lockfiles carry no license
  metadata, so these are reported as "unknown" unless resolved from an
  inventory).
- `package.json` — the package's own SPDX `license`.
- Inventory JSON — an array of `{"name", "version", "license"}` objects, e.g.
  the output of `cargo-license` or an SBOM export.

## Usage

```bash
# Verify against the repository policy
lawsynth-license-check check Cargo.lock inventory.json --policy deny.toml

# Machine-readable output for CI
lawsynth-license-check check inventory.json --format json

# Tolerate dependencies with no recorded license
lawsynth-license-check check Cargo.lock --allow-unknown

# Generate an attribution NOTICE grouped by license
lawsynth-license-check notice inventory.json --out NOTICE.generated
```

## Exit codes

- `0` — all licenses allowed (and no unknowns, unless `--allow-unknown`).
- `1` — at least one denied license, or an unknown license without
  `--allow-unknown`.

## SPDX support

Compound expressions are evaluated: `OR` passes if any branch is allowed, `AND`
requires all operands, `WITH` exceptions and parentheses are recognised.
