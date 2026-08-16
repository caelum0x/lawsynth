# Interventions

Use constant `parameters` or `inputs` in a `Scenario` for a value that applies
throughout a run. Use `parameter_schedule` and `input_schedule` for explicit
time-indexed changes:

```python
Scenario(
    world,
    {"x": 1.0},
    inputs={"u": 0.0},
    input_schedule=[(2.0, "u", 1.0), (4.0, "u", 0.0)],
)
```

Scheduled names are checked against the World: parameter changes may only
target declared parameters, while input changes may only target non-state
variables. Times and values must be finite.
