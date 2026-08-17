# bundle-inspector

Inspect and verify LawSynth `.lsworld` bundles.

A `.lsworld` file is a stored (uncompressed) ZIP archive containing exactly:

1. `manifest.json` — a byte-for-byte fixed contract document
2. `provenance/checksums.sha256` — SHA-256 of every payload entry
3. `world/world.bin` — the binary-v1 world encoding

See `specs/bundle/` and `crates/lawsynth-bundle` for the authoritative format.

## What it does

- Reads the ZIP container and validates entry paths (CRC-32 is checked on read).
- Confirms `manifest.json` matches the fixed contract exactly.
- Recomputes and verifies the SHA-256 checksum manifest, detecting tampering.
- Decodes the world payload (variables, roles, units, parameters, and laws) into
  a readable summary, rendering law expressions from their preorder encoding.

It is a read-only diagnostic. It does not simulate, discover, or mutate bundles.

## Usage

```sh
# Human-readable report (exit 0 = valid, 1 = invalid/tampered)
python src/main.py inspect path/to/world.lsworld

# Machine-readable JSON report
python src/main.py inspect path/to/world.lsworld --json
```

Installed as a package it exposes the `bundle-inspector` console script.

## Development

```sh
python -m pytest tools/bundle-inspector/tests
```

Pure standard library, deterministic, and offline — no Rust toolchain required.
