# Regime states

`Segmentation::label_at` maps an observation index to its segment label. `DiscreteHmm` represents a finite number of hidden states with initial probabilities, a transition matrix, and categorical emission probabilities. Its validation checks dimensions, non-negative finite values, and normalized distributions.

A segment label names an index interval; it does not name a physical mechanism. An HMM state names a configured finite latent category; it does not supply semantic labels or parameter learning.
