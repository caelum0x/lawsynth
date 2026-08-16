# lawsynth-opt

Deterministic small-scale optimizers for calibrating candidate laws. It exposes
coordinate search, L-BFGS, Nelder–Mead, bounded affine least squares, and a
mixed strategy, all with explicit termination reasons.

## Use

```rust
use lawsynth_opt::{LbfgsConfig, ParameterBounds, lbfgs_minimize};

let result = lbfgs_minimize(
    &[2.0], ParameterBounds::new(-10.0, 10.0)?, LbfgsConfig::default(),
    |x| ((x[0] - 1.0).powi(2), vec![2.0 * (x[0] - 1.0)]),
)?;
assert!(result[0] > 0.9 && result[0] < 1.1);
# Ok::<(), lawsynth_opt::OptimizationError>(())
```

Objectives are caller-supplied and must be finite over their accepted domain.
This crate reports convergence or budget termination; it does not manufacture
gradients, enforce scientific identifiability, or replace model validation.
