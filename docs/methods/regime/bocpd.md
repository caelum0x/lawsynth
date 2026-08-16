# BOCPD monitor

`bocpd` maintains a full run-length posterior under a constant hazard and a conjugate Gaussian-mean observation model with caller-supplied observation variance and prior precision. Each output carries the change probability and most likely run length at that index.

The posterior is not truncated, so work and storage grow with the series length. It supports a mean-change model only, assumes finite scalar observations, and does not infer hyperparameters.
