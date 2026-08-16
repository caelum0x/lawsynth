# Intervention boundary

No `do`-operator, structural-equation evaluator, counterfactual engine, or intervention planner is implemented in `lawsynth-causal`. A graph edge and a declared assumption are insufficient to compute an intervention distribution.

Callers must not reinterpret `GrangerResult`, `IndependenceResult`, or `MarkovEquivalence` as the effect of an intervention. Those APIs deliberately expose predictive and structural diagnostics only.

An intervention contract will require explicit treatment semantics, outcome model, adjustment or identification assumptions, and estimand definition; none are silently supplied here.
