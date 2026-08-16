# CLI

Run the CLI through Cargo during development:

```sh
cargo run -p lawsynth-cli -- --help
```

Implemented commands are:

```text
lawsynth inspect WORLD.lsworld
lawsynth discover OBSERVATIONS.csv --time COLUMN --state NAME[,NAME...] --output WORLD.lsworld
lawsynth simulate WORLD.lsworld --initial NAME=VALUE --start T --end T --step DT
lawsynth simulate-discrete WORLD.lsworld --initial NAME=VALUE --steps N [--start T]
```

`discover` accepts polynomial degree and threshold controls, STLSQ or SR3,
optional trigonometric/rational features, one derivative estimator, smoothing,
bootstrap replicates, and symbolic-depth controls. `simulate` and
`simulate-discrete` accept repeated `--initial`, `--parameter`, and `--input`
assignments plus scheduled `TIME:NAME=VALUE` parameter/input changes.

CSV parsing is intentionally strict: quoted CSV fields, missing values, and
non-numeric values are not supported by this local CLI reader. The process
returns status 2 and an explicit diagnostic for invalid inputs or unavailable
commands.
