# Bootstrap

`bootstrap` resamples a non-empty `Samples` collection using `BootstrapConfig { resamples, seed }` and evaluates a caller-provided statistic. It returns the empirical draws and exposes their standard error. Fixed seeds make a run reproducible.

The routine samples scalar observations independently with replacement. It does not implement block bootstrap, stratified bootstrap, paired multivariate resampling, bias correction, or automatic choice of resample count.
