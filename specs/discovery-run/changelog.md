# Discovery-run contract changelog

## Current implementation

- Continuous sparse equation discovery using finite-difference-aligned data.
- Configurable STLSQ or SR3 fitting, polynomial terms, optional trigonometric
  and bounded rational features, and an optional bounded symbolic branch.
- Deterministic preprocessing reports, bootstrap MSE intervals, cancellation,
  and LSCP2 checkpoint resumption at the Rust API.
- CLI CSV ingestion and `.lsworld` output for the first Pareto candidate.

The run contract does not presently define a serialized run manifest, a
checkpoint CLI flag, streaming progress events, or a claim of causal or
stochastic discovery.
