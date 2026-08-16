# LawSynth discovery

`lawsynth-discovery` turns an aligned numeric `Dataset` into one or more
executable continuous-time `World` candidates. Its default path profiles data,
estimates derivatives, constructs a feature matrix, fits STLSQ, and returns a
Pareto front. Optional branches add trigonometric or bounded rational features,
Savitzky–Golay differentiation, deterministic moving-block bootstrap intervals,
and bounded symbolic grammar search.

## Phase 2 recovery budget

The noisy Lorenz and Lotka–Volterra recovery tests in
`src/execute.rs` use 2,001 and 4,001 samples respectively, fixed deterministic
noise, degree-two sparse features, and a 1 ms sampling interval. The target
budget is **under 2 seconds per system in a debug test build on a contemporary
laptop CPU**; they run as ordinary unit tests so regressions are visible in the
workspace test suite.

Run the benchmark-equivalent checks with:

```sh
cargo test -p lawsynth-discovery recovers_recognizable -- --nocapture
```

The model is considered recognizable when the recovered laws evaluate near the
known Lorenz and Lotka–Volterra derivatives at the documented probe states.
