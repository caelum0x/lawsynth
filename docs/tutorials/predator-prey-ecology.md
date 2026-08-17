# A predator–prey ecology walkthrough

This tutorial discovers the classic Lotka–Volterra predator–prey system from
data, reads its laws, runs what-ifs two different ways, compares scenarios, and
exports the result. It's a good tour of the ecology recipe and of LawSynth's
**two intervention semantics**.

## 1. Generate a known system

`lawsynth new` writes a real world bundle **and** a deterministic observation CSV
by simulating the true system — perfect ground truth to check discovery against:

```bash
lawsynth new lotka-volterra --output lv_true.lsworld --data prey.csv --samples 400
```

The true system (prey `x`, predator `y`) is:

```
dx/dt = alpha*x - beta*x*y        (prey grows, is eaten)
dy/dt = delta*x*y - gamma*y       (predators grow by eating, die off)
```

with `alpha=1.1, beta=0.4, delta=0.1, gamma=0.4`, started from `x=10, y=5`.

## 2. Discover with the ecology recipe

Predator–prey coupling is the bilinear `x·y` term. The ecology preset uses a
quadratic library sized exactly for that:

```bash
lawsynth discover prey.csv --time time --state x,y --output prey.lsworld --preset ecology
lawsynth explain prey.lsworld
```

`explain` should recover the growth/decay structure in plain language — e.g. "x
increases in proportion to x", "x decreases in proportion to x·y", and the
mirror terms for `y`.

SDK:

```python
import lawsynth
study  = lawsynth.Study.from_csv("prey.csv", time="time", state=["x", "y"])
result = study.discover(recipe="ecology")
print(result.explain().to_text())     # laws + per-state R²/RMSE
```

## 3. Sanity-check the recovery

Diff the discovered world against the ground-truth template:

```bash
lawsynth compare lv_true.lsworld prey.lsworld
```

Since discovery inlines coefficients as constants while the template carries named
parameters, expect structural equivalence in the laws with the constants matching
the true `alpha/beta/delta/gamma`. Validate the forecast skill too:

```bash
lawsynth validate prey.lsworld --data prey.csv --time time --holdout 0.2
```

## 4. What-ifs, two ways

### (a) SDK: start from a different population (initial-condition override)

The SDK's `forecast`/scenarios override **initial conditions on states** — same
dynamics, different starting point:

```python
fc = result.forecast({"x": 20.0}, horizon=40, step=0.05)   # start with twice the prey
print(fc.divergence)     # |final − baseline| per state

board = (
    study
    .add_scenario("prey_boom",   interventions={"x": 20.0})
    .add_scenario("pred_crash",  interventions={"y": 1.0})
    .compare_scenarios(horizon=40, step=0.05)
)
print(board.table())     # baseline + each scenario, final state + divergence
```

### (b) CLI: change a rate mid-run (scheduled parameter intervention)

The CLI `--intervene`/`--scenario` schedule changes to **named parameters**,
which the *template* world carries (a purely discovered world has none). So run
what-ifs on the template:

```bash
# raise the predation rate beta to 0.6 at t=10
lawsynth forecast lv_true.lsworld --horizon 40 --step 0.05 \
  --initial x=10 --initial y=5 --intervene beta=0.6@10 --output whatif.csv

# compare named rate-change scenarios against baseline, overlaid in HTML
lawsynth scenarios lv_true.lsworld --horizon 40 --step 0.05 \
  --initial x=10 --initial y=5 \
  --scenario over_hunting:beta=0.6@10 \
  --scenario recovery:gamma=0.2@10,delta=0.05@20 \
  --html scenarios.html
```

> This is the key distinction to remember: **SDK what-ifs move the starting
> point; CLI `--intervene` changes a parameter/input on the fly.** Use the SDK on
> discovered worlds, and the CLI's scheduled interventions on template (or
> otherwise parameterized) worlds.

## 5. Put honest bands on a forecast

Bootstrap the model's residuals against the observed window:

```bash
lawsynth forecast prey.lsworld --horizon 40 --step 0.05 --initial x=10 --initial y=5 \
  --confidence --data prey.csv --time time --level 0.9 --seed 7 --html bands.html
```

Or, for uncertainty that comes from *model structure*, use the ensemble band:

```python
ens = study.discover_ensemble(n=16, fraction=0.8, seed=0, recipe="ecology")
band = ens.forecast(horizon=40, lower_q=0.1, upper_q=0.9)
```

## 6. Share and take it elsewhere

```bash
lawsynth report prey.lsworld --data prey.csv --time time --output prey.report.html
lawsynth export prey.lsworld --format python --output prey.py
lawsynth export prey.lsworld --format latex               # the \dot{x}=… law system
lawsynth library add prey.lsworld --name predator-prey --tags ecology --from-data prey.csv
```

In a notebook, `result.explore()` (after `enable_explore()`) opens an interactive
widget where you can drag the initial prey/predator populations and watch the
cycle change in real time.

## See also

- [Forecast and run what-ifs](../cookbook/04-forecast-and-whatifs.md) — the intervention-semantics box in depth.
- [Pick the right settings per domain](../cookbook/03-domain-presets.md)
