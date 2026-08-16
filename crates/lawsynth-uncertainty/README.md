# lawsynth-uncertainty

`lawsynth-uncertainty` provides deterministic, dependency-free primitives for
quantifying uncertainty in a discovered law. It validates observations before
calculation and returns explicit errors for undefined analyses.

It includes empirical bootstrap distributions and percentile intervals,
unbiased covariance estimation, first-order (delta-method) propagation,
independent empirical Monte-Carlo propagation, local quadratic profile fitting,
and explainable structural uncertainty aggregation.

```rust
use lawsynth_uncertainty::{bootstrap, BootstrapConfig, Samples};

let data = Samples::new(vec![1.0, 2.0, 3.0])?;
let result = bootstrap(&data, BootstrapConfig::default(), |x| {
    x.iter().sum::<f64>() / x.len() as f64
})?;
# let _ = result;
# Ok::<(), lawsynth_uncertainty::UncertaintyError>(())
```

Monte-Carlo propagation treats input sample columns as independent. Preserve
observed dependence by using `CovarianceMatrix` with `linear_propagate`, or
resample joint rows in application code.
