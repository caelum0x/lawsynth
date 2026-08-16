# Transitions

`TransitionMatrix::from_states` counts adjacent observed state labels and normalizes each row that has outgoing observations. It exposes individual transition probabilities. `DiscreteHmm::viterbi` uses a supplied transition matrix and categorical emissions to return the highest-scoring state path.

Rows with no observed outgoing transitions remain zero in the empirical matrix. Treat such rows as evidence of missing transition data, not as a calibrated absorbing-state model.

The crate does not estimate continuous-time transition rates or covariate-dependent transitions.
