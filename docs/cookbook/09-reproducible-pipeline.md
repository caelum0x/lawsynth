# Automate a reproducible pipeline

**Goal:** drive the whole loop — ingest → discover → (optionally) validate →
report → export — from **one config file**, so the same config and data always
reproduce the same worlds, reports, and summary. Deterministic and offline.

## Get a starting config

```bash
lawsynth pipeline --example > pipeline.toml
```

This prints a documented, ready-to-run config. It uses a small hand-rolled
sections-plus-`key = value` reader (with `#`/`;` comments and `["a","b"]`
arrays) — no external TOML crate.

## The config

```toml
# LawSynth pipeline config
[input]
csv = "observations.csv"    # ingest this CSV (.csv/.tsv/.parquet)
time = "time"               # name of the time column
state = ["x", "y"]          # state columns to discover laws for

[discovery]
degree = 2                  # polynomial feature degree
threshold = 0.05            # sparse coefficient threshold
solver = "stlsq"            # stlsq | sr3
trigonometric = false       # add sin/cos features
rational = false            # add rational features
regimes = false             # segment the primary state into regimes
pareto = false              # report Pareto frontier size
refine = false              # joint parameter refinement
causal = false              # dependency/causal hypothesis

[validate]                  # optional: omit the whole section to skip validation
holdout = 0.2               # fraction held out (by time) to score forecast skill

[outputs]
world = "model.lsworld"        # required: the discovered world bundle
report = "model.report.html"   # self-contained HTML report (with residuals)
title = "My model"             # optional report title
export_python = "model.py"     # optional: runnable python module
export_latex = "model.tex"     # optional: LaTeX law system
```

`[input].csv`, `[input].state`, and `[outputs].world` are required; everything
else has sensible defaults. Only `python` and `latex` exports are wired into the
pipeline — for the other formats, run `lawsynth export` on the produced bundle
([recipe 7](07-export-model.md)).

## Run it

```bash
lawsynth pipeline pipeline.toml
```

**Expected shape:**

```
pipeline: observations.csv
  discovered 2 state law(s): mse=<e>, complexity=<n>
  validate: STRONG - the model tracks held-out data closely
  artifacts:
    - model.lsworld
    - model.report.html
    - model.py
    - model.tex
```

If you omit the `[validate]` section, the line reads
`validate: skipped (no [validate] section)`. With `pareto = true`, a
`pareto frontier: <k> candidate(s)` line is added.

## Python SDK equivalent

There is no single "run this TOML" SDK call — instead compose the same steps
explicitly, which is itself fully reproducible:

```python
import lawsynth

study  = lawsynth.Study.from_csv("observations.csv", time="time", state=["x", "y"])
result = study.discover(degree=2, threshold=0.05)   # ingest -> discover

result.report("model.report.html")                  # report
result.save("model.lsworld")                         # bundle

# validate on a holdout via the SDK backtest ([recipe 5]) or the CLI:
#   lawsynth validate model.lsworld --data observations.csv --holdout 0.2
```

For a one-command reproducible artifact set, the CLI `pipeline` is the intended
surface; the SDK is the intended surface for programmatic, notebook-driven work.

## See also

- [Clean noisy data first](02-clean-noisy-data.md) — prep before the pipeline if needed.
- [How much can I trust it?](05-trust-validation.md) — what the `[validate]` verdict means.
