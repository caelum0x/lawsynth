# Quickstart: the core loop end to end

This walks the full **discover → understand → use → share** loop on a two-state
system. Substitute your own columns for `x,y`.

## 1. Prepare observations

Create a numeric CSV with a header, a strictly increasing finite `time` column, and
one or more finite state columns. For example `observations.csv`:

```csv
time,x,y
0,1,0
0.05,0.998,0.099
0.10,0.990,0.197
```

CSV, TSV, and a numeric Parquet subset are accepted (`.csv`, `.tsv`, `.parquet`).

## 2. Discover the law system

```sh
lawsynth discover observations.csv \
  --time time --state x,y --output world.lsworld
```

`discover` writes a portable `.lsworld` bundle and prints the fit (mean squared
error) and complexity. Useful options: `--degree N`, `--threshold VALUE`,
`--solver stlsq|sr3`, `--trigonometric`, `--rational`, one derivative estimator
(`--spline`, `--spectral`, `--savgol-window ODD_N`, or `--tvreg-lambda VALUE`),
`--smooth-radius N`, `--bootstrap REPLICATES`, `--symbolic-depth N`, and the
optional stages `--regimes`, `--pareto`, `--refine`, `--causal`.

## 3. Understand what it found

```sh
lawsynth explain world.lsworld
lawsynth inspect world.lsworld
```

`explain` prints the readable laws, their dominant terms, the dependency structure,
and the assumptions a result is contingent on. `inspect` reports state, variable,
and parameter counts and distinguishes continuous from discrete worlds.

## 4. Use it: simulate and forecast

```sh
# Simulate over a fixed window
lawsynth simulate world.lsworld --initial x=1 --initial y=0 \
  --start 0 --end 10 --step 0.05

# Forecast beyond the observed window
lawsynth forecast world.lsworld --horizon 20 --step 0.05 --output forecast.csv
```

Interventions are first-class and *scheduled*: `--parameter-at TIME:NAME=VALUE` and
`--input-at TIME:NAME=VALUE` on `simulate`, and `--intervene NAME=VALUE@TIME` on
`forecast`. Each targets a **parameter or a non-state input** at a given time — for
example `--intervene beta=0.2@5` on an SIR world.

## 5. Compare

Diff two worlds directly:

```sh
lawsynth compare world.lsworld other.lsworld --html compare.html
```

Or overlay named what-if scenarios (each a set of scheduled interventions) against a
baseline over one horizon:

```sh
lawsynth scenarios world.lsworld --horizon 20 --step 0.05 \
  --scenario mitigated:beta=0.2@5 --scenario surge:beta=0.6@5 --html scenarios.html
```

The Python `Study` API also offers scenario boards keyed by *initial-condition*
overrides (`study.add_scenario("hot", interventions={"x": 2.0})`) — see
[python.md](python.md).

## 6. Share

```sh
lawsynth report world.lsworld --output report.html
lawsynth export world.lsworld --format python --output world_model.py
```

`report` is a single self-contained HTML file — rendered equations, fit, regime
timeline, uncertainty bands, and inline SVG charts, with no server or external
assets. `export` emits a dependency-free Python module, a LaTeX `align*` block, or a
documented JSON description.

Discovery is deterministic for identical input and options: rerun any step and you
get exactly the same world, report, and forecast.
