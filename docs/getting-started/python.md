# Python

The Python SDK wraps the same native engine as the CLI. Build the native extension
in place from `python/lawsynth`:

```sh
python -m pip install maturin
maturin develop
python -m pytest -q tests
```

The pure-Python data and configuration classes import without a native build;
discovery, simulation, and bundle IO resolve `lawsynth._native` lazily and raise a
clear error if it is missing.

## `Study` — the whole loop, fluently

`Study` collapses `observe → discover → understand → use → compare → share` into a
few lines. Every returned object renders richly in a Jupyter notebook.

```python
import lawsynth

# 1. Ingest observations into a validated dataset and build a study
study = lawsynth.Study.from_csv("observations.csv", time="time", state=["x", "y"])

# 2. Discover the executable world
result = study.discover()                 # -> DiscoveryResult

# 3. Understand
print(result.explain())                   # -> Explanation (readable laws, fit, deps, assumptions)

# 4. Use: simulate and what-if forecast
traj = study.simulate(horizon=20, step=0.05)
forecast = study.forecast({"x": 1.5}, horizon=20)   # baseline vs. counterfactual + divergence

# 5. Compare scenarios
study.add_scenario("hot", interventions={"x": 2.0})
study.add_scenario("cold", interventions={"x": 0.5})
comparison = study.compare_scenarios()    # -> ScenarioComparison

# 6. Share
study.report("report.html")               # self-contained HTML report
study.save("world.lsworld")               # portable bundle the CLI/Studio also read
```

Construct a study from other inputs with `Study.from_dataset(...)`,
`Study.from_columns(...)`, or `Study.load(path, dataset=..., state=...)` to rebind a
persisted world. Tune discovery by passing a `DiscoveryConfig` or keyword overrides
to `discover()` (`polynomial_degree`, `threshold`, `solver`, `include_trigonometric`,
`include_rational`, `smoothing_radius`, `derivative_method`, `symbolic_depth`).

## Importing from external sources

`Study.from_source(...)` (and the lower-level `lawsynth.load_source(...)`) bring in
observations through the `lawsynth-connectors` package — `filesystem`, `http`, `s3`,
`postgres`, and `sqlite` connectors — coercing records to finite floats at the SDK
boundary:

```python
study = lawsynth.Study.from_source(
    "filesystem", "obs.csv",
    time="t", state=["x", "y"], options={"root": "."},
)
study.discover().explain()
```

## Lower-level API

For direct control, call `lawsynth.discover(...)` on aligned columns:

```python
from lawsynth import discover

world = discover(
    time=[0.0, 0.05, 0.10],
    columns={"x": [1.0, 0.998, 0.990], "y": [0.0, 0.099, 0.197]},
    state=["x", "y"],
)
print(world.equations())
```

## Notebook dashboard

With the optional `lawsynth-notebook` package installed, `study.dashboard()` (and
`result.dashboard()`) renders a cohesive `StudyDashboard` — equations, dependency
graph, trajectory, uncertainty, and any registered scenarios folded together.

Use only finite numeric observations and valid identifiers throughout.
