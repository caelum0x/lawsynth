# STLSQ

`stlsq` repeatedly solves ridge-regularized least squares on the currently active columns, drops coefficients whose magnitude is below `threshold`, and stops when the active mask stabilizes or `max_iterations` is reached. The returned solution includes full-width coefficients, active indices, and residual sum of squares.

The linear systems use deterministic Gaussian elimination with pivoting. Singular or malformed designs are errors; no pseudoinverse, cross-validation, or automatic threshold tuning is substituted.
