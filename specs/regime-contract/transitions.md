# Empirical transitions

`TransitionMatrix::from_states(states, state_count)` counts adjacent observed state pairs in a `state_count × state_count` matrix. Every supplied state index must be below `state_count`, and `state_count` must be positive.

Rows with one or more outgoing transitions are normalized to conditional frequencies. Rows with no outgoing transition contain only zeroes, so they are deliberately not probability distributions. `probability(from, to)` returns `None` for out-of-range indices.

This matrix is descriptive. It does not estimate uncertainty, apply pseudocounts, or guarantee ergodicity.
