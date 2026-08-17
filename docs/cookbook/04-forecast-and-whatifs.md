# Forecast and run what-ifs

**Goal:** simulate a world forward, ask "what if?", and put honest uncertainty
bands around the forecast.

> ## Two intervention semantics — read this first
>
> - **CLI `--intervene NAME=VALUE@TIME`** (forecast) and **`--scenario
>   NAME:k=v@t`** (scenarios) schedule a change to a **parameter or non-state
>   input** at time `t`. **Template** worlds (`lawsynth new`) carry named
>   parameters; **discovered** worlds do **not** (coefficients are inlined
>   constants), so `--intervene` on a discovered world errors with *"intervention
>   target is neither a parameter nor a non-state input."* On a discovered world,
>   change the **starting point** with `--initial` instead.
> - **SDK `Study.forecast(...)` / `add_scenario(...)`** override **initial
>   conditions on state variables** — same dynamics, different starting point.

## CLI: point forecast

```bash
# forecast a world 40 time units ahead from a chosen starting state
lawsynth discover prey.csv --time time --state x,y --output prey.lsworld
lawsynth forecast prey.lsworld \
  --horizon 40 --step 0.05 \
  --initial x=10 --initial y=5 \
  --output forecast.csv
```

Defaults if omitted: `--horizon 20`, `--start 0`, `--step 0.1`, and each state
starts at `1.0`. **Expected shape** (with `--output`, only the summary prints):

```
wrote forecast: forecast.csv (<rows> rows)
forecast horizon t in [0, 40], <rows> samples
final state:
  x                10 -> <float>
  y                5 -> <float>
```

Without `--output`, the trajectory CSV is printed first, then the summary.

## CLI: scheduled what-ifs (`--intervene` / `scenarios`)

These need **named parameters**, so use a template world:

```bash
lawsynth new lotka-volterra --output lv.lsworld

# one forecast with a scheduled parameter change: raise beta to 0.6 at t=10
lawsynth forecast lv.lsworld --horizon 40 --step 0.05 \
  --initial x=10 --initial y=5 \
  --intervene beta=0.6@10 \
  --output whatif.csv

# compare several named scenarios against an implicit baseline
lawsynth scenarios lv.lsworld --horizon 40 --step 0.05 \
  --initial x=10 --initial y=5 \
  --scenario hunt:beta=0.6@10 \
  --scenario recover:gamma=0.2@10,delta=0.05@20 \
  --html scenarios.html
```

`scenarios` prints a comparison table — final state per variable and its signed
divergence from baseline — and, with `--html`, overlays every scenario on one
chart per state:

```
Scenarios over t in [0, 40], step 0.05 (<n> samples); world lv.lsworld
2 scenario(s) + baseline, 2 state variable(s)

scenario   interventions        x       dx      y       dy
baseline   (baseline)           <..>    -       <..>    -
hunt       beta=0.6@10          <..>    +<..>   <..>    -<..>
recover    gamma=0.2@10, ...    <..>    -<..>   <..>    +<..>

wrote scenario report: scenarios.html (<bytes> bytes)
```

## CLI: confidence bands (`--confidence`)

Bands are estimated by **bootstrapping the model's residuals on observed data**,
so they need `--data` — without observations there is nothing to bound:

```bash
lawsynth forecast prey.lsworld \
  --horizon 40 --step 0.05 --initial x=10 --initial y=5 \
  --confidence --data prey.csv --time time \
  --level 0.9 --replicates 512 --seed 7 \
  --output bands.csv --html bands.html
```

Defaults: `--level 0.95`, `--replicates 512`, a fixed `--seed`. **Expected
shape:**

```
wrote forecast bands: bands.csv (<rows> rows)
confidence forecast t in [0, 40], <rows> samples, 90% band from residual bootstrap (512 replicates, seed 0x...)
  state            residuals   offset_lower   offset_upper      offset_se
  x                     <n>        <e>            <e>             <e>
  y                     <n>        ...
wrote band report: bands.html (<bytes> bytes)
```

The band CSV columns are `time, x_lower, x_median, x_upper, y_lower, …`. The
whole estimate is deterministic for a fixed seed.

## Python SDK: what-ifs are initial-condition overrides

```python
import lawsynth

study  = lawsynth.Study.from_csv("prey.csv", time="time", state=["x", "y"])
result = study.discover()

# forecast: start y from a different initial value, compare to baseline
fc = result.forecast({"y": 8.0}, horizon=40, step=0.05)   # -> Forecast
print(fc.divergence)          # {'x': |Δx_final|, 'y': |Δy_final|}
print(fc.interventions)       # {'y': 8.0}  (the initial-condition override)
# fc.baseline and fc.counterfactual are TrajectoryData
```

Only **state variables** may be intervened on; naming anything else raises a
`ValidationError` listing the valid states.

### Compare many scenarios (scenario board)

```python
comparison = (
    study
    .add_scenario("more_prey",   interventions={"x": 20.0})
    .add_scenario("fewer_pred",  interventions={"y": 2.0})
    .compare_scenarios(horizon=40, step=0.05)
)
print(comparison.table())     # baseline + each scenario, final state + divergence
comparison.distance("more_prey")   # Euclidean norm of final-state divergence
```

`add_scenario` returns `self`, so scenarios chain fluently; the baseline
(no-intervention) run is always implicit. In a notebook, `comparison` renders as
an overlaid multi-series chart per state plus a divergence table.

For **ensemble** forecast bands (uncertainty from model structure, not just
residuals), see [recipe 5](05-trust-validation.md).

## See also

- [How much can I trust it?](05-trust-validation.md)
- [Use LawSynth as a service](10-service-and-jupyter.md) for the `Client.forecast` endpoint.
