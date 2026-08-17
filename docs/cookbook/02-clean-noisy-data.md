# Clean noisy data before discovery

**Goal:** understand your data's quality *before* modeling it, then clean it so
finite-difference derivatives (which discovery relies on) aren't dominated by
noise. The loop is `profile → prep → discover`.

Noisy or unevenly sampled data degrades the derivative estimates discovery uses,
so a light clean-up often recovers a materially better fit.

## CLI

### 1. Profile — know your data

```bash
lawsynth profile raw.csv --time time
```

**Expected shape:**

```
dataset profile: raw.csv
  rows:        <n>
  columns:     2  (x, y)
  fingerprint: 0x...

time column 'time':
  range:       0.000000e0 .. 1.995000e1
  step:        5.000000e-2 (uniform)
  ordering:    strictly increasing (monotonic)
  regular:     yes

columns:
  name          type       count  missing  ...  min  max  mean  std
  x             numeric      <n>        0   ...
  ...

warnings:
  - column 'y' has <k> Tukey-IQR outlier(s)
```

Add `--json` for machine-readable output. Warnings flag too-few samples,
irregular sampling, constant/degenerate columns, outliers, and missing values.

### 2. Prep — apply real transforms, in order

`prep` operations apply **in the order given**, each on the previous result:

```bash
lawsynth prep raw.csv --time time --output clean.csv \
  --trim 0.5:18.0 \
  --drop-constant \
  --smooth-window 3 \
  --resample 0.05
```

| Flag | Effect |
|------|--------|
| `--trim START:END` | keep only the usable time window `[START, END]` |
| `--drop-constant` | remove columns that never change (zero range) |
| `--detrend` | subtract a least-squares linear trend per column |
| `--smooth-window N` | centered moving average, `N` = half-window radius (≥1) |
| `--resample DT` | linearly re-grid every column onto a uniform `DT` step |

**Expected shape** — a provenance summary with content fingerprints:

```
prep raw.csv -> clean.csv
  input : <n> rows, 2 column(s) [x,y]
  op    : trim [0.5, 18] kept <k> of <n> rows
  op    : smooth-window radius=3 (moving average), fingerprint ... -> ...
  op    : resample dt=0.05 -> <m> uniform samples, fingerprint ... -> ...
  output: <m> rows, 2 column(s) [x,y]
  change: rows <n> -> <m>, columns 2 -> 2, content fingerprint ... -> ...
```

### 3. Discover on the cleaned data

```bash
lawsynth discover clean.csv --time time --state x,y --output clean.lsworld
```

To *show* the improvement, discover on both and compare the fit in each report
(`lawsynth report ... --data ...`), or validate both (see
[recipe 5](05-trust-validation.md)) and compare the R² / skill scores.

## Python SDK

`Study.profile()` and `Study.prepare()` are pure-standard-library and
deterministic. `prepare()` returns a **new** study on a cleaned copy — the
original is untouched.

```python
import lawsynth

study = lawsynth.Study.from_csv("raw.csv", time="time", state=["x", "y"])

# 1. profile
report = study.profile()
print(report.to_text())
print(report.warnings)          # tuple of plain-language quality warnings

# 2. prepare -> a new, cleaned study (applied in order: trim, resample, smooth, detrend)
clean = study.prepare(
    trim=(0.5, 18.0),
    resample_dt=0.05,
    smooth=3,                   # moving-average window in samples
    # detrend=True or detrend=["x"] to remove a per-column linear trend
)

# 3. discover on each and compare the fit
raw_fit   = study.discover().explain().fit
clean_fit = clean.discover().explain().fit
print("raw   x R²:", raw_fit["x"]["r_squared"])
print("clean x R²:", clean_fit["x"]["r_squared"])
```

`prepare(columns=[...])` restricts *smoothing and detrending* to named columns;
trimming and resampling always apply to the whole dataset so the time axis stays
consistent.

> The SDK's `smooth` is a window **width** in samples; the CLI's `--smooth-window`
> is a half-window **radius**. Both are centered moving averages.

## See also

- [Discover from a CSV](01-discover-from-csv.md) once the data is clean.
- [Automate a reproducible pipeline](09-reproducible-pipeline.md) to bake this into one config.
