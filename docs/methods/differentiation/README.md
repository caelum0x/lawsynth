# Differentiation

`lawsynth-differentiate` turns aligned numeric samples into an aligned derivative dataset. Every method returns one derivative per input sample and retains the source column identifier and unit. It is deterministic and rejects mismatched lengths, inadequate sample counts, and method-specific invalid grids.

Available estimators are finite difference, local quadratic Savitzky–Golay, natural cubic spline, direct-DFT spectral differentiation, and total-variation denoising followed by finite difference. Choosing an estimator is a modeling decision: no method infers an observation model, repairs missing data, or supplies uncertainty intervals.
