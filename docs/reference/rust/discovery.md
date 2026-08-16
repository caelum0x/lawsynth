# `lawsynth-discovery`

`discover(dataset, config)` executes the continuous discovery pipeline: validate inputs, estimate derivatives, generate configured features, fit sparse laws, score candidates, and construct an executable world. `DiscoveryConfig` names the state columns and controls polynomial degree, sparse solver, feature families, differentiation, smoothing, optional bootstrap, and optional bounded symbolic search.

The pipeline is deterministic for a fixed dataset/configuration. It is not an oracle for causal structure or arbitrary symbolic mathematics: unsupported data semantics and invalid numerical configurations return typed errors rather than fabricated models.
