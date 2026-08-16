# Numerical stability contract

The Granger implementation solves normal equations with partial pivoting. `CausalConfig::singular_tolerance` is finite and strictly positive; a pivot whose magnitude is at most this tolerance returns `SingularDesign` rather than yielding coefficients from an ill-conditioned design.

Pearson independence similarly returns `SingularDesign` for a constant input series. It does not regularize, jitter, or discard columns.

These guards prevent undefined calculations, but they are not a general condition-number analysis. Users needing robust regression or statistical inference should use a dedicated estimator and record its assumptions separately.
