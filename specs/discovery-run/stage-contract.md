# Stages

`DiscoveryPlan::from_config` exposes this fixed stage ordering:

1. Validate
2. Preprocess
3. Profile
4. Differentiate
5. GenerateFeatures
6. FitLaws
7. Score
8. Finalize

The executor validates limits/state membership before preprocessing. It
profiles the working dataset before differentiation, removes the first and
last feature rows to align finite-difference targets, fits each requested
state, then creates and Pareto-filters candidates.

`lawsynth-core::ProgressStage` and `ProgressTracker` define a general
monotonic-per-stage progress event primitive, but the current discovery
executor does not emit a progress stream. Consumers must not infer runtime
event delivery merely from the plan enum.
