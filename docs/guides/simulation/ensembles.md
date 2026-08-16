# Ensembles

The native simulator runs one deterministic scenario per invocation. It has no ensemble runner, distribution type, random-number stream, or calibrated uncertainty output.

To study sensitivity, generate an explicit table of initial states or overrides in an external workflow, run each scenario in an isolated directory, and retain the scenario ID with each CSV. Use fixed seeds only for the external sampler and publish the sampling distribution and bounds.

Do not describe a grid of deterministic runs as probabilistic uncertainty unless the sampling design and model-error assumptions support that interpretation. Ensemble aggregation, confidence bands, and stochastic dynamics require additional, separately validated tooling.
