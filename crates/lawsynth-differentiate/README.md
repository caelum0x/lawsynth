# lawsynth-differentiate

Deterministic derivative estimation for equation discovery. Supported methods
include finite differences, Savitzky–Golay smoothing, spectral differentiation,
natural cubic splines, total-variation regularization, irregular three-point
estimates, and weak-form integrals.

## Use

```rust
use lawsynth_differentiate::differentiate_series;

let derivative = differentiate_series(&[0.0, 1.0, 2.0, 3.0], &[0.0, 1.0, 4.0, 9.0])?;
assert_eq!(derivative.len(), 4);
# Ok::<(), lawsynth_differentiate::DifferentiationError>(())
```

Choose a method through `DerivativeConfig` and retain that configuration with
results. Derivatives amplify noise and edge estimates are less reliable; this
crate exposes methods, not a claim that a derivative is directly observed.
