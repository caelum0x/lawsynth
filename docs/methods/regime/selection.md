# Selecting a regime model

Use penalized segmentation when a scalar series is plausibly piecewise constant in mean. Use BOCPD for an online mean-change signal with fixed hyperparameters. Use Viterbi only when discrete HMM probabilities are already supplied.

Penalty, hazard, and state count are user decisions. The crate provides no held-out selection, marginal likelihood comparison, or automatic calibration.
