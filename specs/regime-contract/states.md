# Discrete state models

`DiscreteHmm` represents a fully specified finite HMM: an initial distribution, square transition matrix, and one emission distribution per state. `validate` requires a positive state count, a positive common symbol count, exact dimensions, finite non-negative probabilities, and each distribution sum within `1e-9` of one.

`viterbi(observations)` runs maximum log-probability decoding for valid symbol indices. If no state can emit an observation at a step, it returns `ImpossibleObservation { index }`; empty sequences and out-of-range symbols are rejected.

There is no parameter learning, smoothing, forward-backward posterior, continuous emission model, or hidden-state confidence output.
