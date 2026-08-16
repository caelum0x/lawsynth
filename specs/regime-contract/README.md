# Regime contract

`lawsynth-regime` implements deterministic segmentation and finite-state primitives for scalar, finite observations. The public algorithms are exact penalized least-squares segmentation, exhaustive binary splitting, a compact BOCPD filter, discrete-HMM Viterbi decoding, transition counts, and regime-indexed affine laws.

The crate does not decode mixed continuous/discrete state spaces, infer nonlinear laws, or provide a live event engine. Invalid observations and malformed probability models return `RegimeError` rather than being repaired.
