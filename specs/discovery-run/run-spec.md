# Run specification

A run receives a finite dataset and `DiscoveryConfig`. The config requires a
nonempty list of existing state identifiers and supplies polynomial degree,
optional trigonometric/rational terms, sparse solver/configuration,
derivative method, optional moving-average radius or ordered preprocessing
pipeline, optional bootstrap configuration, optional symbolic configuration,
and resource limits.

Execution validates resource limits and inputs, applies preprocessing, profiles
the resulting data, differentiates it, evaluates features, fits one sparse law
per state, constructs a continuous `World`, optionally evaluates a symbolic
candidate, and Pareto-filters candidates by MSE and AST complexity. There must
be at least three observations.

The CLI accepts only comma-delimited numeric CSV with a header:

```text
lawsynth discover OBSERVATIONS.csv --time time --state x,y --output world.lsworld
```

It also accepts degree, threshold, `stlsq|sr3`, feature flags, one derivative
method selection, smoothing, bootstrap replicate count, and symbolic depth.
It writes the first Pareto candidate as an `.lsworld` bundle and prints its MSE
and complexity. CSV parsing is intentionally simple: quoted fields, embedded
commas, nullable cells, and units are not supported by this command.
