# Structure map integration

A structure map is a view of model dependencies, not evidence of a causal graph. Generate graph nodes and edges from an inspected equation representation, preserve edge direction and term context, and label the result as a dependency visualization.

Use renderer-neutral graph data from `@lawsynth/chart-core` where it matches the host needs. The host renderer owns keyboard navigation, focus order, colors, hit testing, and screen-reader descriptions; chart-core deliberately contains none of those browser concerns.

LawSynth does not currently infer causal structure, latent variables, interventions from observational data, or graph uncertainty. Avoid causal-language labels unless supplied by an independently validated causal workflow.
