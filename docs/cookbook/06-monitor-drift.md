# Watch a live system for drift

**Goal:** treat a discovered world as a *model of normal behaviour* and score a
stream of fresh observations against it. When the system stops tracking the
model, you want to know — with the exact timestamps that broke.

`monitor` simulates the world across the new data's window (seeded from the first
observed row), forms the per-state residual `observed − predicted`, standardizes
it with a **robust** median/MAD scale, and flags any timestamp whose standardized
residual exceeds `K` sigma. The robust scale means a *sustained* shock stands out
instead of inflating the very statistic meant to catch it.

## CLI

```bash
lawsynth monitor prey.lsworld --data fresh.csv --time time --threshold 3
```

Defaults: `--time time`, `--threshold 3`. **Expected shape:**

```
Monitor: prey.lsworld against fresh.csv
  window t in [<..>, <..>] over <n> observation(s), threshold K=3

  state        mean_resid    rms_resid   max|resid|   ctrl_scale     max|z|  flagged
  x               <e>          <e>          <e>          <e>          <..>        0
  y               ...

  no anomalous timesteps
Verdict: IN-CONTROL - observations stay within the model's expected spread
```

When observations drift, flagged timestamps are listed and the verdict escalates:

```
  <k> anomalous timestep(s) at t = <t1>, <t2>, ...
Verdict: DRIFT DETECTED - <p>% of timesteps breach the control limit; the system is no longer tracking the model
```

The verdict is `IN-CONTROL` (no flags), `ANOMALIES FLAGGED` (<5% of timesteps),
or `DRIFT DETECTED` (≥5%).

## Python SDK

`Study.monitor(new_dataset, threshold=3.0)` returns a `MonitorReport`. Build the
fresh dataset however you like — from a CSV via another `Study`, or directly:

```python
import lawsynth

# discover the model of "normal" once
study  = lawsynth.Study.from_csv("prey.csv", time="time", state=["x", "y"])
study.discover()

# score a fresh batch of observations against it
fresh = lawsynth.Study.from_csv("fresh.csv", time="time", state=["x", "y"]).dataset
report = study.monitor(fresh, threshold=3.0)   # -> MonitorReport

print(report.to_text())
print(report.in_control)         # True / False
print(report.verdict)            # 'in control' | 'out of control — N anomalies flagged'
report.flagged_times()           # timestamps where at least one state was flagged
for a in report.anomalies:
    print(a.time, a.state, a.z, a.observed, a.simulated)
```

**Expected shape** of `to_text()`:

```
Monitor report — prey
  verdict: IN CONTROL (threshold = 3 sigma, <n> samples)

Per-state residuals:
  state      n        mean         std    robust σ    max|z|  flagged
  x       <n>       <..>        <..>        <..>      <..>        0
  y       ...

No anomalies — observations track the model within threshold.
```

You can also call the function directly: `lawsynth.monitor(world, dataset,
state=[...], threshold=3.0)`. In a notebook the report renders a standardized-
residual chart with the ±threshold envelope drawn in, so exceedances are obvious.

Everything is deterministic and offline: the same world and data always produce
the same report; shock-injected data flags the anomaly at the injected timestamp.

## See also

- [How much can I trust it?](05-trust-validation.md) — validate before you deploy.
- [Monitoring a running system](../tutorials/monitoring-a-running-system.md) — the full narrative.
