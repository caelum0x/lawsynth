# lawsynth-score

Deterministic scoring primitives for ranking discovery candidates. It computes
residual and fit summaries, expression complexity, information criteria,
selection stability, dimensional compatibility, Pareto fronts, and weighted
candidate ranks.

## Use

```rust
use lawsynth_score::fit_statistics;

let fit = fit_statistics(&[1.0, 2.0, 3.0], &[1.1, 1.9, 3.0])?;
assert!(fit.mean_squared_error >= 0.0);
# Ok::<(), lawsynth_score::ScoreError>(())
```

Scores make trade-offs inspectable but do not turn observational fit into a
unique causal explanation. Rank models only over comparable samples and retain
the configuration and residual evidence alongside every selected candidate.
