# Trajectories

`TrajectoryData.from_native(trajectory)` copies native `time` and per-state `values` into immutable tuples. Construction requires nonempty strictly increasing time, at least one value series, and series lengths equal to the time length. `column(name)` returns a series or raises `ValidationError` for an unknown name.

`TrajectoryData` is a validated result container. It does not interpolate, resample, plot, export, evaluate event roots, or add uncertainty bands.
