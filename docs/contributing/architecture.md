# Architecture contributions

The implemented execution line is:

```text
CSV -> data/profile/derivative/features -> sparse discovery -> World -> bundle -> simulation
```

Keep validation at boundaries. `lawsynth-core` owns common identifiers,
hashing, diagnostics, cancellation, and resources; expression, units, data,
World, simulator, bundle, and discovery crates compose the local engine. The
CLI and Python extension call those same crates rather than duplicating their
semantics.

Changes to a World, expression, or bundle format require compatibility tests
and explicit version handling. The current canonical bundle encodes scalar
continuous/discrete Worlds only. Do not claim support for events, regimes,
stochastic terms, delays, custom calls, signatures, or causal metadata until
the IR, serialization, validation, and executor all implement it.
