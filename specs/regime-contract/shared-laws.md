# Shared-law boundary

The implemented law book stores one affine scalar law per regime. It contains no shared-parameter representation, hierarchy, shrinkage penalty, or multi-regime fitting algorithm.

If two labels need the same law, callers may insert equal coefficients explicitly. That is value equality, not a shared mutable or statistical parameter object.

Claims about common structure across detected regimes require a model and fit procedure outside this crate.
