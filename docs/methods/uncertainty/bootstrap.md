# Empirical bootstrap

`bootstrap` samples a validated scalar vector with replacement for a configured number of replicates and applies a caller-provided statistic to each draw. It uses a seeded SplitMix64 generator and rejection sampling for indices, making results reproducible for the same inputs and callback.

The observations are treated as exchangeable scalar samples. This is not a block bootstrap, residual bootstrap, Bayesian bootstrap, or time-series-respecting resampler.
