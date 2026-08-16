# lawsynth-regime

`lawsynth-regime` detects changes in the mean of a finite scalar series and
works with finite-state discrete HMMs. `pelt` implements the exact penalized
least-squares dynamic-programming objective with a minimum segment length.
`bocpd` produces online Gaussian change probabilities, and `DiscreteHmm`
provides validated Viterbi decoding.

The scalar change-point model is intentionally narrow: it does not silently
claim multivariate, heteroscedastic, or causal regime identification.
