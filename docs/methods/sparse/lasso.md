# Lasso

`lasso` uses cyclic coordinate descent for the squared-error objective with an L1 penalty. The coefficient update uses soft thresholding after temporarily restoring the coordinate's residual contribution.

The API validates a non-negative finite penalty and uses a caller-specified iteration limit. It does not standardize automatically, compute a regularization path, screen features, or report duality gaps.
