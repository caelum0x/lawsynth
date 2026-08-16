# SR3

`sr3` uses an auxiliary sparse vector and alternates a ridge-like coefficient update with hard thresholding of that auxiliary vector. `SparseConfig` supplies the threshold, ridge strength, and maximum iteration count.

This is a deterministic relaxation routine for a single dense response vector. It has no continuation schedule, convergence certificate, group/proximal variants, or automatic regularization selection.
