# Failure cases

Derivative estimation amplifies noise. Sparse regression can then select terms that compensate for estimation error, especially when candidate columns are correlated. A low derivative residual can still yield a poor long-horizon rollout. Test at least one held-out rollout and inspect residual structure rather than accepting a single aggregate score.

Periodic spectral differentiation assumes the sequence is periodic; endpoint discontinuity contaminates all frequencies. Natural splines impose endpoint curvature, finite differences use one-sided endpoints, and Savitzky--Golay requires a valid local window. Select these assumptions from the acquisition process, not from the most attractive equation.

The system rejects non-finite values, unequal column lengths, and non-increasing time. It cannot diagnose whether a finite but biased sensor, unrecorded input, or changing regime invalidates an inferred law. Record those risks as exclusions in the result.
