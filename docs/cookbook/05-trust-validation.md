# How much can I trust it?

**Goal:** move past "it fits the data it was trained on" to real evidence — does
the world *forecast* held-out data, and is its *structure* stable? Three
complementary checks:

1. **Holdout validation** — split by time, forecast the tail, score the skill.
2. **Rolling-origin backtest** — forecast from many origins, watch skill decay.
3. **Ensemble term stability** — which terms are robust vs. sampling artifacts.

## 1. Holdout validation (`validate`)

### CLI

```bash
lawsynth validate prey.lsworld --data prey.csv --time time --holdout 0.2
```

Splits observations into train `[0, split)` and holdout `[split, n)` by time,
simulates the world across the holdout from the split point, and scores per state
with RMSE, MAE, R², and a **skill score vs. a persistence baseline**. Defaults:
`--time time`, `--holdout 0.2` (0.05–0.9 allowed). **Expected shape:**

```
Validation: prey.lsworld on prey.csv
  split at t=<..> | train=<..> rows | holdout=<..> rows (fraction 0.20)

  state             RMSE          MAE        R2  skill_vs_persist
  x            <e>          <e>           0.99xx           0.9xxx
  y            ...

  aggregate  R2=0.99xx  skill=0.9xxx
Verdict: STRONG - the model tracks held-out data closely
```

Verdicts range STRONG / GOOD / FAIR / WEAK, and note when a model *doesn't beat a
persistence baseline*.

### Python SDK

The SDK expresses holdout-style evaluation through the richer **rolling-origin
backtest** below (which subsumes a single holdout by evaluating from many
origins). For a single train/tail split, use the CLI `validate` above, or the
`pipeline` `[validate]` section ([recipe 9](09-reproducible-pipeline.md)).

## 2. Rolling-origin backtest

The "does it *forecast*?" check: it picks `origins` evenly spaced forecast
origins, simulates forward `horizon` **observation steps** from each, scores
against what actually happened, and builds a skill-vs-horizon decay curve.
Available from both the CLI and the SDK.

### CLI

```sh
lawsynth backtest prey.lsworld --data prey.csv --origins 5 --horizon 40 \
  --html backtest.html
```

Prints per-state RMSE/MAE/R² pooled across origins, the mean-error-vs-horizon
decay, and a `STRONG | MODERATE | WEAK` verdict; `--html` writes a self-contained
skill-vs-horizon report.

### SDK

```python
import lawsynth

study  = lawsynth.Study.from_csv("prey.csv", time="time", state=["x", "y"])
result = study.discover(recipe="ecology")

bt = study.backtest(origins=5, horizon=40)   # horizon in observation steps
print(bt.to_text())
print(bt.verdict)          # 'strong forecasting skill' | 'moderate' | 'weak' | ...
print(bt.mean_r_squared)   # mean R² across states
print(bt.decay)            # error(H) / error(1); 1.0 == forecasts as well far out
```

**Expected shape** of `to_text()`:

```
Backtest — prey
  rolling-origin walk-forward · 5 origins · horizon 40 step(s) · states: x, y

Aggregate forecast accuracy (out-of-sample, across all origins):
  state           RMSE         MAE        R²
  x             <..>        <..>    0.9xxx
  y             ...

Skill vs. horizon (mean |error| across origins & states):
  h=1     h=2     ...
  <..>    <..>    ...

Verdict: strong forecasting skill (mean R² = 0.9xxx; error grows 1.4x from lead 1 to 40).
```

In a notebook `bt` renders a skill-vs-horizon chart plus accuracy and per-origin
tables. `horizon` defaults to a window that leaves room for the origins if
omitted.

> **Which trust check?** `validate` is a single train/tail holdout; `backtest`
> generalizes it to many rolling origins and shows how skill decays with horizon.
> Both ship on the CLI and the SDK.

## 3. Ensemble term stability (SDK)

A single discovery yields one set of coefficients but says nothing about how
*stable* that structure is. `discover_ensemble` re-discovers on `n` deterministic
bootstrap resamples and reports, per law term, its **selection frequency** and
coefficient **mean/std** — separating robust terms from artifacts.

```python
ens = study.discover_ensemble(n=16, fraction=0.8, seed=0, recipe="ecology")
print(ens.to_text())

ens.robust_terms()        # terms selected in ≥80% of members with tight spread
ens.consensus_laws()      # readable laws from terms selected in ≥50% of members
band = ens.forecast(horizon=40, lower_q=0.1, upper_q=0.9)   # ensemble forecast band
```

**Expected shape** of `to_text()`:

```
Ensemble uncertainty — prey
  16 members (requested 16) · <m>/<n> rows each · seed=0 · fraction=0.8

Term stability (per law term across resamples):
  target  term         select%        mean         std
  x       x·y             100%      <..>        <..>    robust
  x       x                94%      <..>        <..>
  y       y                88%      <..>        <..>
  ...

Consensus laws (terms selected in >= 50% of members):
  dx/dt = ...
  dy/dt = ...

Robust terms: <k> of <t> observed (x<-x·y, ...).
```

Because resample indices derive purely from `seed` (never the clock), the whole
ensemble reproduces bit-for-bit. Each member keeps rows in time order (an
`m`-of-`n` draw without replacement), so the time axis stays valid.

## Reading the three together

- **validate** high but **backtest decay** steep → fits the window, extrapolates
  poorly. Trust short forecasts only.
- **validate** high and **ensemble** shows unstable terms → the fit may hinge on
  a term that's an artifact of this sample; collect more data or raise the
  threshold.
- All three strong → a genuinely trustworthy world.

## See also

- [Forecast and run what-ifs](04-forecast-and-whatifs.md) for residual bands.
- [Watch a live system for drift](06-monitor-drift.md) once deployed.
