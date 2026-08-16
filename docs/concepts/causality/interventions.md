# Interventions

World simulation accepts parameter and control interventions at specified finite times. A `SimulationRequest` applies them as scheduled values, allowing counterfactual-style execution under an explicitly modified model.

That operation answers a model-implied scenario question. Its external causal validity depends on the world structure, parameterization, and assumptions supplied by the user.

The causal crate does not implement do-calculus, adjustment-set search, mediation analysis, or population transport.
