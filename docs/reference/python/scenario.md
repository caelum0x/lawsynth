# Scenarios and trajectory data

`Scenario(initial, parameters={}, inputs={}, interventions=())` is an immutable simulation request. Mapping keys must be identifiers. `Intervention(time, target, value, kind)` permits finite values and `kind` of `parameter` or `input`; interventions are sorted by their dataclass order when a scenario is created.

`simulate(world, scenario, start=0.0, end=1.0, step=0.01)` creates the native request and returns `TrajectoryData`. It represents scheduled constant overrides, not event triggers, state resets, intervention optimization, or causal analysis.
