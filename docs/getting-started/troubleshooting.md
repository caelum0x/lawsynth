# Troubleshooting

**Cargo cannot download a crate.** This repository sets Cargo offline mode.
Populate the locked dependency cache in an approved networked environment,
then return to offline mode. Do not use `cargo update` as a workaround.

**`lawsynth._native` cannot be imported.** Build the extension from
`python/lawsynth` with `maturin develop` using the Python interpreter that
runs the application. The pure-Python package imports, but native discovery
and simulation correctly remain unavailable without that build.

**Discovery rejects my CSV.** Supply a header, the exact `--time` column name,
equal-length numeric columns, finite values, and strictly increasing times.
The CLI reader intentionally accepts only simple comma-separated numeric
records; preprocess richer files before invoking it.

**A bundle will not simulate.** Use `inspect` first. Continuous simulation
requires a continuous bundle and all state initial values; discrete simulation
requires a discrete bundle and `--steps`.
