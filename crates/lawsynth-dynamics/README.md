# lawsynth-dynamics

Dataset-backed system-identification problem definitions. It packages validated
continuous, discrete, delayed, controlled, and implicit observation problems
and exposes transition extraction and central derivative refinement.

## Use

```rust
use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_dynamics::ContinuousProblem;

let data = Dataset::new(TimeAxis::new(vec![0.0, 1.0, 2.0])?,
    vec![NumericColumn::new("x", vec![1.0, 2.0, 4.0])?])?;
let problem = ContinuousProblem::new(data, vec![Identifier::new("x")?])?;
assert_eq!(problem.state().len(), 1);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Problems preserve the supplied data and state selection. They do not choose a
feature library or fit a law; use them as validated inputs to discovery methods.
