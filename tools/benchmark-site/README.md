# benchmark-site

Render LawSynth's scientific benchmark results into a static site.

The `benchmarks/` tree holds the checked-in discovery benchmarks. Each case is a
directory with a `benchmark.toml` (id, title, capability status), an
`expected.json` (the expected observable status), a `baseline.json`, and,
after a run, a `score.json` recording the observed status. This tool reads those
declarative files and produces a reviewable summary. It never executes the
benchmarks, so it stays deterministic and offline.

## What it does

- **Load** — collects every `benchmark.toml` case under a benchmarks directory.
- **Classify** — compares each case's observed status against its expectation,
  distinguishing pass, fail, regression, pending, and capability-boundary.
- **Render** — emits a static `index.html` (with an inline dependency-free SVG
  status chart and a per-benchmark table) plus a machine-readable `results.json`.

The command exits non-zero when any regression or failure is present, so it
doubles as a CI gate.

## Usage

```sh
# Build the site into ./site (exit 1 if any regression/failure)
python src/main.py build benchmarks --out site

# Print just the results.json summary to stdout
python src/main.py build benchmarks
```

Installed as a package it exposes the `benchmark-site` console script.

## Development

```sh
python -m pytest tools/benchmark-site/tests
```
