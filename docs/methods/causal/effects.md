# Effect estimation boundary

No average treatment effect, adjustment-set search, do-calculus, instrumental-variable estimator, or counterfactual estimator is implemented in `lawsynth-causal`. The graph API can represent an assumed acyclic structure; it does not execute interventions or identify effects.

Use the simulation layer for explicit parameter/input scenarios only after independently supplying a valid world model. That is mechanistic scenario evaluation, not observational effect identification.
