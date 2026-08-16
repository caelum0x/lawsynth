# lawsynth-sim

Deterministic simulation of continuous and discrete World IR. The crate
compiles world laws, evaluates explicit contexts, integrates ODEs with RK4,
steps discrete worlds, splits hybrid intervals at events, and offers a seeded
Euler–Maruyama path for diagonal-noise SDE experiments.

## Use

```rust
use lawsynth_sim::{SimulationConfig, SimulationRequest};

let config = SimulationConfig::new(0.0, 10.0, 0.01)?;
let request = SimulationRequest::default();
assert!(config.step > 0.0);
let _ = request;
# Ok::<(), lawsynth_sim::SimulationError>(())
```

`simulate` validates states, parameters, controls, finite evaluation, step
budgets, and event timing. Numerical results are approximations; report solver
settings and compare convergence when scientific conclusions depend on them.
