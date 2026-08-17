# lawsynth-dataset-registry

`lawsynth-dataset-registry` indexes and verifies the scientific benchmark
datasets checked in under `benchmarks/`. Each benchmark case declares itself in a
`benchmark.toml` (id, title, version, and a capability contract). The registry
records that metadata plus SHA-256 checksums of the declarative files, so an
index can be verified for drift later and datasets can be resolved by id.

Following `specs/reproducibility/data-hash.md`, the numeric series are generated
on demand from `benchmark.toml` rather than stored. The registry therefore
hashes the *declarative* files (`benchmark.toml`, `expected.json`,
`baseline.json`, `README.md`) that fully determine each case. The tool is
dependency-free and performs no network I/O.

## Usage

```bash
# Index every benchmark case under a root into a registry file
lawsynth-dataset-registry index benchmarks --out datasets.registry.json

# Verify the registry against the files on disk
lawsynth-dataset-registry verify datasets.registry.json --root benchmarks

# Render a Markdown datasheet for one dataset
lawsynth-dataset-registry card datasets.registry.json dynamics/ode-small

# Stage a dataset's declarative files into a directory (local copy, no network)
lawsynth-dataset-registry stage datasets.registry.json dynamics/ode-small ./staged --root benchmarks
```

## Registry format

```json
{
  "registry_version": "0.1",
  "datasets": [
    {
      "id": "dynamics/ode-small",
      "title": "Small continuous ODE",
      "version": 1,
      "capability": "supported",
      "path": "dynamics/ode-small",
      "files": [
        { "path": "benchmark.toml", "sha256": "…", "bytes": 492 }
      ]
    }
  ]
}
```

## Exit codes

- `index`, `card`, `stage` — `0` on success, `2` for an unknown dataset id.
- `verify` — `0` when all checksums match, `1` when any file is missing or
  changed.

## Boundaries

The registry catalogues and verifies datasets; it does not run discovery or
regenerate series. Materialising the numeric data is the job of the benchmark
generators under `benchmarks/`.
