# CLI

Install the `lawsynth` binary, or run it in place during development:

```sh
cargo install --path crates/lawsynth-cli
# or
cargo run -p lawsynth-cli -- --help
```

Every command prints its own usage. The full command set, grouped by the product
loop:

## Discover

```text
lawsynth discover OBS.{csv,tsv,parquet} --time COLUMN --state NAME[,NAME...] --output WORLD.lsworld
  [--degree N] [--threshold VALUE] [--solver stlsq|sr3] [--trigonometric] [--rational]
  [--savgol-window ODD_N | --spline | --spectral | --tvreg-lambda VALUE [--tvreg-iterations N]]
  [--smooth-radius N] [--bootstrap REPLICATES] [--symbolic-depth N]
  [--regimes] [--pareto] [--refine] [--causal]
```

Only one derivative estimator may be selected at a time. `--regimes`, `--pareto`,
`--refine`, and `--causal` turn on the corresponding optional discovery stages and
add their findings to the printed summary.

## Understand

```text
lawsynth explain WORLD.lsworld
lawsynth inspect WORLD.lsworld
```

## Use

```text
lawsynth simulate WORLD.lsworld --initial NAME=VALUE [--initial ...] --start T --end T --step DT
  [--parameter NAME=VALUE] [--input NAME=VALUE]
  [--parameter-at TIME:NAME=VALUE] [--input-at TIME:NAME=VALUE]
lawsynth simulate-discrete WORLD.lsworld --initial NAME=VALUE [--initial ...] --steps N [--start T] ...
lawsynth forecast WORLD.lsworld [--horizon T] [--start T] [--step DT]
  [--initial NAME=VALUE]... [--parameter NAME=VALUE]... [--intervene NAME=VALUE@TIME]... [--output FORECAST.csv]
```

## Compare

```text
lawsynth compare WORLD-A.lsworld WORLD-B.lsworld [--json] [--html FILE]
lawsynth scenarios WORLD.lsworld [--horizon T] [--start T] [--step DT]
  [--initial NAME=VALUE]... --scenario NAME[:k=v@t,...] [--scenario ...] [--html FILE]
```

## Share

```text
lawsynth report WORLD.lsworld [--output REPORT.html] [--title TEXT]
  [--start T] [--end T] [--step DT] [--initial NAME=VALUE]... [--data OBS.{csv,tsv,parquet}] [--time COLUMN]
lawsynth export WORLD.lsworld --format <python|latex|json> [--output FILE]
```

## Organize, scaffold, and validate

```text
lawsynth library <add|list|show|remove> [--dir DIR] ...
lawsynth templates
lawsynth new TEMPLATE [--output WORLD.lsworld] [--data OBS.csv] [--samples N]
lawsynth validate WORLD.lsworld --data OBS.{csv,tsv,parquet} [--time COLUMN] [--holdout FRACTION]
lawsynth pipeline PIPELINE.toml | lawsynth pipeline --example
lawsynth doctor
```

`library` maintains a local index (default `~/.lawsynth/library.tsv`, override with
`--dir`) of tagged, described `.lsworld` bundles. `templates`/`new` scaffold from the
built-in worlds `lorenz`, `lotka-volterra`, `pendulum`, `van-der-pol`, and `sir`.
`validate` scores a world on a held-out fraction of a dataset. `pipeline` runs a
whole ingest → discover → validate → report → export flow from one TOML file
(`--example` prints a documented sample). `doctor` reports whether the install is
healthy.

CSV parsing is intentionally strict: quoted fields, missing values, and non-numeric
values are rejected. On invalid input or an unknown command the process exits with a
non-zero status and an explicit diagnostic.
