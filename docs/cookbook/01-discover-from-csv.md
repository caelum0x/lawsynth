# Discover a model from a CSV

**Goal:** turn a `time, …` CSV of observations into an executable world, read what
it found in plain language, and produce a shareable HTML report — the core
`discover → explain → report` loop.

## Ingredients

A CSV with a numeric time column and one column per state variable:

```
time,x,y
0.0,10.0,5.0
0.05,10.42,4.71
...
```

Don't have one? Generate a deterministic sample:

```bash
lawsynth new lotka-volterra --data prey.csv --samples 400
```

## CLI

```bash
# 1. discover: fit sparse laws for states x and y
lawsynth discover prey.csv \
  --time time --state x,y \
  --output prey.lsworld

# 2. explain: plain-language + structured reading of the world
lawsynth explain prey.lsworld

# 3. report: a single self-contained HTML file (laws, charts, fit overlay)
lawsynth report prey.lsworld --data prey.csv --time time --output prey.report.html
```

`discover` requires `--time`, `--state`, and `--output`. Inputs may be `.csv`,
`.tsv`, or `.parquet`. Passing `--data` to `report` overlays the observed samples
on the simulated trajectory and adds a residual strip, so you can *see* the fit.

**Expected shape** — `discover` prints a one-line summary:

```
discovered world: mse=<float>, complexity=<n>
```

`explain` prints a structured block:

```
World summary
  2 state variable(s), 2 variable(s) total, 0 parameter(s)
  dimensionality: 2-dimensional  |  total complexity: <n> AST node(s)

Laws
  dx/dt = ...
    - x increases in proportion to ...
    reads: x, y
  ...

Variables
  x                state      [dimensionless]
  ...
```

`report` writes the file and confirms:

```
wrote report: prey.report.html (<bytes> bytes, 2 state variable(s))
overlaid observations for 2 state(s)
```

> Note: a **discovered** world has **0 parameters** — coefficients are inlined as
> constants. That matters for CLI what-ifs (see
> [recipe 4](04-forecast-and-whatifs.md)).

## Python SDK

```python
import lawsynth

study = lawsynth.Study.from_csv("prey.csv", time="time", state=["x", "y"])

result = study.discover()                 # -> DiscoveryResult
print(result.explain().to_text())         # plain-language + fit (R², RMSE)

result.report("prey.report.html")         # self-contained HTML
result.save("prey.lsworld")               # portable bundle
```

`Study.from_csv` validates the CSV at the boundary (missing columns, non-numeric
rows fail fast with a clear `ValidationError`). `discover()` returns a
`DiscoveryResult` whose `.explain()` gives a structured `Explanation`:

```text
Study: prey
Observed 400 samples over t ∈ [0, 19.95]
State variables: x, y

Discovered laws:
  dx/dt = ...
      dominant term: x·y
  ...

Fit quality (simulation vs. observations):
  x: R² = 0.99xx, RMSE = ...
  y: R² = 0.99xx, RMSE = ...
```

### Tune when the default under-fits

The default is polynomial degree 2 with sparsity threshold 0.05. If discovery
misses terms, raise the degree or lower the threshold:

```bash
lawsynth discover prey.csv --time time --state x,y --output prey.lsworld \
  --degree 3 --threshold 0.02
```

```python
result = study.discover(degree=3, threshold=0.02)
```

For domain-aware defaults, jump to [recipe 3](03-domain-presets.md).

## See also

- [Clean noisy data first](02-clean-noisy-data.md) if the fit is poor.
- [How much can I trust it?](05-trust-validation.md) to validate before you rely on it.
