# LawSynth air-gapped bundle

A self-contained, offline install bundle for the LawSynth production stack. It
lets you run LawSynth on a host with **no network access** by shipping the
container images, a Python wheelhouse, reference datasets, and the production
compose profile as one verifiable package.

This directory holds the **bundle tooling** (scripts + content lists). The
actual bundle is produced by `export.sh` on a connected host and transferred to
the air-gapped target.

## Workflow

```
  connected host                 reviewed transfer            air-gapped host
 ┌───────────────┐                                          ┌──────────────────┐
 │  export.sh    │  ──►  lawsynth-airgap-<ver>/  ──►  USB/  │  verify.sh       │
 │  pull+save    │        images/ wheels/ datasets/   disk  │  import.sh       │
 │  wheels       │        compose/ manifest.yaml            │  install.sh      │
 │  checksums    │        checksums.sha256                  │  compose up      │
 └───────────────┘                                          └──────────────────┘
```

1. **Export** (connected): `BUNDLE_VERSION=0.1.0 ./export.sh` pulls the images
   in `images.txt`, builds/downloads the wheels in `packages.txt`, gathers the
   datasets in `datasets.txt`, copies the production compose profile, stamps
   `manifest.yaml`, and writes `checksums.sha256` over everything. Add
   `ARCHIVE=1` to also emit a single `.tar.gz`.
2. **Transfer** the produced `dist/lawsynth-airgap-<ver>/` (or the archive)
   over your approved offline channel.
3. **Verify** (air-gapped): `./verify.sh` recomputes and checks every hash.
4. **Import** (air-gapped): `./import.sh` `docker load`s the images and stages
   the wheelhouse and datasets.
5. **Install** (air-gapped): `./install.sh` verifies, imports, renders `.env`
   from the template, validates the compose config, and starts the stack with
   `--pull never` so it never contacts a registry.

`install.sh` runs steps 3–5 for you; you only need `export.sh` on the connected
side and `install.sh` on the offline side.

## Files

| File | Role |
|---|---|
| `images.txt` | Container images to save (kept in sync with compose/production) |
| `packages.txt` | Python packages for the operator wheelhouse |
| `datasets.txt` | Reference datasets (`<dest> <repo:...\|https://...>`) |
| `manifest.yaml` | Bundle schema + layout; stamped at export time |
| `checksums.sha256` | Integrity hashes (placeholder until exported) |
| `export.sh` | Produce a bundle on a connected host |
| `verify.sh` | Integrity-check a bundle |
| `import.sh` | Load images, stage wheels/datasets on the offline host |
| `install.sh` | Verify + import + configure + start the stack |

## Requirements

- **Connected host:** docker, `python3 -m pip`, tar, curl, network access.
- **Air-gapped host:** docker + docker compose (or `docker-compose`).
- Same CPU architecture on both ends — set `TARGET_PLATFORM` at export time
  (default `linux/amd64`) to match the target.

## Security notes

- **Secrets are never bundled.** `install.sh` copies `.env.example` to `.env`;
  you must fill in every REQUIRED secret before the stack accepts traffic.
- **Integrity is your responsibility.** `checksums.sha256` detects corruption
  and accidental tampering; for provenance, sign the transferred archive out of
  band and verify the signature before `install.sh`.
- Only ship datasets you are licensed to redistribute.
- There is no online update feed. To upgrade, produce a new bundle and repeat
  the workflow. See `docs/self-hosting/airgap.md` and `.../upgrade.md`.
