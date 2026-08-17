# Monitoring a running system

Once you've discovered a world, it becomes a **model of normal behaviour**. This
tutorial builds that baseline model, then scores fresh batches of observations
against it to catch drift — and shows how to reason about *why* the system moved.

The story: you run a process instrumented with two channels `x` and `y`. You have
a clean historical window (the "known good" regime) and a stream of new data
arriving in batches. You want an automatic, deterministic verdict on each batch.

## 1. Learn "normal"

Discover the world from the known-good window:

```bash
lawsynth discover baseline.csv --time time --state x,y --output normal.lsworld --preset general
lawsynth validate normal.lsworld --data baseline.csv --time time --holdout 0.2
```

Only promote a model to "the definition of normal" if it validates well — a model
that doesn't track its own held-out data can't judge anything else. SDK:

```python
import lawsynth
baseline = lawsynth.Study.from_csv("baseline.csv", time="time", state=["x", "y"])
baseline.discover()
```

## 2. Score a fresh batch

`monitor` simulates the world across the new window (seeded from its first row),
forms per-state residuals `observed − predicted`, standardizes them with a
**robust** median/MAD scale, and flags any timestamp beyond `K` sigma.

```bash
lawsynth monitor normal.lsworld --data batch_001.csv --time time --threshold 3
```

A healthy batch:

```
Monitor: normal.lsworld against batch_001.csv
  window t in [...] over <n> observation(s), threshold K=3
  ...
  no anomalous timesteps
Verdict: IN-CONTROL - observations stay within the model's expected spread
```

A drifting batch names the offending timestamps and escalates the verdict:

```
  <k> anomalous timestep(s) at t = <t1>, <t2>, ...
Verdict: DRIFT DETECTED - <p>% of timesteps breach the control limit; the system is no longer tracking the model
```

The verdict tiers are `IN-CONTROL`, `ANOMALIES FLAGGED` (<5% of timesteps), and
`DRIFT DETECTED` (≥5%). The robust scale means a *sustained* shift is caught
rather than quietly inflating the control limit.

## 3. Automate the batch loop (SDK)

```python
import lawsynth

for name in ("batch_001.csv", "batch_002.csv", "batch_003.csv"):
    fresh = lawsynth.Study.from_csv(name, time="time", state=["x", "y"]).dataset
    report = baseline.monitor(fresh, threshold=3.0)
    print(name, "→", report.verdict)
    if not report.in_control:
        print("   flagged at:", report.flagged_times())
        for a in report.anomalies:
            print(f"   t={a.time:.3g} {a.state}: observed {a.observed:.3g} vs simulated {a.simulated:.3g} (z={a.z:+.2f})")
```

`report.to_dict()` gives you a JSON-friendly structure to ship to a dashboard or
alerting system. In a notebook, `report` renders a standardized-residual chart
with the ±threshold envelope drawn in, so an exceedance is visible at a glance.

## 4. Tune the sensitivity

`--threshold`/`threshold=` is your signal-to-noise dial. Lower it (e.g. `2.5`) to
catch subtler drift at the cost of more false positives on noisy channels; raise
it for only the clearest breaks. Because the whole computation is deterministic,
the same batch and threshold always yield the same flags — so a threshold you
tune on historical incidents behaves identically in production.

## 5. From "something changed" to "what changed"

Monitoring tells you the system left the model's expected envelope. To understand
*how*, re-discover on the drifted batch and diff the worlds:

```bash
lawsynth discover batch_003.csv --time time --state x,y --output drifted.lsworld --preset general
lawsynth compare normal.lsworld drifted.lsworld
```

`compare` shows added/removed/changed laws and parameters and a complexity delta —
often revealing which term's coefficient shifted. You can then explore the
consequences with a what-if:

```python
# e.g. if a rate appears to have changed, explore starting-condition sensitivity
drifted = lawsynth.Study.from_csv("batch_003.csv", time="time", state=["x", "y"])
drifted.discover()
print(drifted.forecast({"x": drifted.dataset.columns["x"][0] * 1.2}, horizon=20).divergence)
```

## What you built

A validated model of normal behaviour, a deterministic per-batch drift verdict
with exact flagged timestamps, an automatable SDK loop, and a diff-based path from
"something changed" to "here's the term that moved."

## See also

- [Watch a live system for drift](../cookbook/06-monitor-drift.md) — the compact recipe.
- [How much can I trust it?](../cookbook/05-trust-validation.md) — validate before you monitor.
