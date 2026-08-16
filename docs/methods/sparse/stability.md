# Stability selection

`stability_selection` requires a positive number of replicates and a sample fraction in `(0, 1]`. Each bootstrap sample has `ceil(n × fraction)` rows and is solved independently with STLSQ.

Rows are sampled independently with replacement. For ordered trajectories this breaks temporal dependence, so use it as a reproducibility diagnostic rather than as a causal or time-series uncertainty procedure.
