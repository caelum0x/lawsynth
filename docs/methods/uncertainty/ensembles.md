# Independent-input Monte Carlo

`monte_carlo_propagate` samples each `Samples` input independently with replacement, evaluates a supplied scalar model, and returns the validated output samples. Its draw count and seed are explicit.

Independent column resampling discards cross-input correlation and temporal structure. Use covariance-aware or joint sampling outside this function when those dependencies matter.
