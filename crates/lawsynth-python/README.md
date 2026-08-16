# lawsynth-python

PyO3 bindings that expose a compact Python-facing World and Scenario API while
reusing the Rust data, discovery, simulation, and bundle implementations. The
extension defines the `lawsynth` module when built with the Python feature set.

## Use

```python
import lawsynth

world = lawsynth.World(
    states=["x"], parameters={"rate": 0.5},
    equations={"x": "rate * x"},
)
trajectory = world.simulate(initial={"x": 1.0}, end=1.0, step=0.01)
assert trajectory.values["x"][-1] > 1.0
```

Boundary helpers validate names, finite numeric values, schedules, and bundle
errors before mapping them to `ValueError`. Python receives deterministic
results for a fixed configuration; it does not bypass the scientific limits of
the underlying numerical model.
