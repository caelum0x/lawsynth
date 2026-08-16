# Sparse regression

LawSynth exposes two sparse solver choices: `stlsq` and `sr3`. Both are selected explicitly with `--solver`; `--threshold` controls term pruning. Smaller thresholds generally retain more terms, while larger thresholds produce simpler equations but can omit weak dynamics.

Use a grid selected before fitting, assess trajectory error and equation complexity jointly, and choose the simplest model that meets the validation criterion. Do not compare only derivative-space training error: integration can amplify small coefficient errors.

The threshold is not a probability, significance level, or universal physical constant. Bootstrap replicates can be requested with `--bootstrap N`, but their interpretation depends on the data-generating process and resampling design; they do not turn a correlation into a causal claim.
