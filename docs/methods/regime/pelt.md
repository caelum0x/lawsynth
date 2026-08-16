# Penalized segmentation

`pelt` minimizes a penalized sum of within-segment squared errors with a minimum segment length. It uses exact dynamic programming over all admissible previous boundaries and reconstructs contiguous segments with means and SSE values.

Despite the module name, implementation intentionally avoids pruning: runtime is quadratic in the number of samples in the worst case. It segments mean changes only; it does not fit trends, variances, multivariate signals, or symbolic dynamics per segment.
