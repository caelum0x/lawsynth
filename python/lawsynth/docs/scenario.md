# Scenario

`lawsynth.Scenario` binds a `World` to its initial state, constant parameter
overrides, control inputs, and optional scheduled changes.

```python
from lawsynth import Scenario, World

world = World(
    ["x"],
    {"rate": 1.0},
    {"x": "rate"},
)
scenario = Scenario(
    world,
    {"x": 0.0},
    parameter_schedule=[(0.5, "rate", 3.0)],
)
trajectory = scenario.simulate(end=1.0, step=1.0)
```

Each schedule entry is `(time, name, value)`. Changes take effect at their
timestamp; continuous integration splits a step at that boundary, and
discrete updates apply a change to the update beginning at that time.
