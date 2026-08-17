# From a messy sensor CSV to a trusted, shareable model

This walkthrough takes a noisy, unevenly sampled two-channel sensor log all the
way to a validated, exported, registered model. It chains: **profile → prep →
discover → explain → validate → backtest → ensemble → report → export →
library**. Everything is deterministic and offline.

We'll assume your log looks like this — a `time` column and two measured
channels `x` and `y`:

```
time,x,y
0.00,10.03,4.98
0.04,10.41,4.70
0.11,10.79,4.55     # note: irregular timestamps
...
```

If you want a concrete file to follow along with, generate a clean synthetic one
and treat it as your "sensor" (then imagine noise on top):

```bash
lawsynth new lotka-volterra --data sensor.csv --samples 400
```

## 1. Know your data first

Never model data you haven't looked at.

```bash
lawsynth profile sensor.csv --time time
```

Read the warnings block carefully — it flags too-few samples, **irregular
sampling**, constant/degenerate columns, outliers, and missing values. Irregular
timestamps and noise both corrupt the finite-difference derivatives discovery
relies on, so if you see either, clean before modeling.

SDK equivalent:

```python
import lawsynth
study = lawsynth.Study.from_csv("sensor.csv", time="time", state=["x", "y"])
print(study.profile().to_text())
```

## 2. Clean it — in a deliberate order

Trim the unusable warm-up, drop dead channels, smooth the noise, and re-grid onto
a uniform step (operations apply in the order given):

```bash
lawsynth prep sensor.csv --time time --output clean.csv \
  --trim 0.5:18.0 \
  --drop-constant \
  --smooth-window 3 \
  --resample 0.05
```

The summary reports row/column counts and content fingerprints before and after,
so the transformation is auditable.

SDK equivalent — `prepare()` returns a *new* study on cleaned data; the original
is untouched:

```python
clean = study.prepare(trim=(0.5, 18.0), resample_dt=0.05, smooth=3)
```

## 3. Discover — and prove cleaning helped

Discover on both raw and cleaned data and compare the fit:

```python
raw_fit   = study.discover(recipe="ecology").explain().fit
clean_fit = clean.discover(recipe="ecology").explain().fit
for s in ("x", "y"):
    print(s, "raw R²", raw_fit[s]["r_squared"], "→ clean R²", clean_fit[s]["r_squared"])
```

CLI:

```bash
lawsynth discover clean.csv --time time --state x,y --output clean.lsworld --preset ecology
```

## 4. Read what it found

```bash
lawsynth explain clean.lsworld
```

`explain` gives a plain-language sentence per term ("x increases in proportion to
x·y", …), the variables/parameters, and the dimensionality/complexity. In the SDK,
`clean.discover(...).explain().to_text()` also prints per-state R² and RMSE.

## 5. Can I trust it? Three independent checks

**Holdout skill** (CLI):

```bash
lawsynth validate clean.lsworld --data clean.csv --time time --holdout 0.2
```

Look for a STRONG/GOOD verdict and a skill score that beats the persistence
baseline.

**Out-of-sample forecast decay** (SDK rolling-origin backtest):

```python
bt = clean.backtest(origins=5, horizon=40)
print(bt.verdict, "· mean R²", round(bt.mean_r_squared, 3), "· decay", round(bt.decay, 2))
```

**Structural stability** (SDK ensemble): which terms are robust vs. sampling
artifacts?

```python
ens = clean.discover_ensemble(n=16, fraction=0.8, seed=0, recipe="ecology")
print(ens.to_text())
print("robust terms:", [f"{t.target}<-{t.feature}" for t in ens.robust_terms()])
```

If validation is strong, backtest decay is gentle, and the ensemble's key terms
are robust, you have a trustworthy world. If the ensemble shows a load-bearing
term flickering in and out, collect more data or raise the threshold before you
rely on it.

## 6. Share it — report, export, register

A self-contained HTML report with the fit overlay and residual strip:

```bash
lawsynth report clean.lsworld --data clean.csv --time time --output clean.report.html
```

Run-anywhere code exports (no LawSynth needed at the destination):

```bash
lawsynth export clean.lsworld --format python --output clean.py
lawsynth export clean.lsworld --format c      --output clean.c
lawsynth export clean.lsworld --format latex             # to stdout
```

Register the bundle with provenance so you can find it later and know exactly
which data and config produced it:

```bash
lawsynth library add clean.lsworld --name sensor-model \
  --tags ecology,production --from-data clean.csv \
  --config "ecology preset, smoothed+resampled" --note "validated, R²>0.9"
```

## 7. Make the experiment reproducible

Track the discovery run so re-running the same config on the same data resolves
to the same content-addressed id:

```bash
lawsynth discover clean.csv --time time --state x,y --output clean.lsworld \
  --preset ecology --track --label sensor-baseline
lawsynth runs list
```

Or fold the whole thing into one `pipeline.toml`
([cookbook recipe 9](../cookbook/09-reproducible-pipeline.md)):

```bash
lawsynth pipeline --example > sensor.pipeline.toml
# edit csv/state/outputs, add a [validate] section, then:
lawsynth pipeline sensor.pipeline.toml
```

## What you built

A cleaned dataset, a discovered world you can read, three independent trust
checks, a shareable report, portable code exports, and a provenance-tracked entry
in your library — all reproducible from the same inputs.

## See also

- [Predator–prey ecology walkthrough](predator-prey-ecology.md)
- [Monitoring a running system](monitoring-a-running-system.md)
