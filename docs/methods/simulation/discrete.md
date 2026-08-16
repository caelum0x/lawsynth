# Discrete dynamics

`simulate_discrete` evaluates every law from an immutable snapshot of the current state, so updates are simultaneous rather than order-dependent. It emits the initial sample and then one sample per requested integer step; scheduled changes at a step time are applied before that update.

Discrete simulation has no fractional clock, event root finding, or automatic stability analysis. Expressions must evaluate to finite values or the run returns an error.
