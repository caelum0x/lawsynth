# Causal contract

`lawsynth-causal` is a deterministic library for validating directed acyclic graphs and computing limited time-series and association diagnostics. Its results are evidence and graph invariants, not automatically identified causal effects.

Implemented components are DAG management, declared assumptions, skeleton/collider signatures, Pearson correlation, lag construction, Granger F statistics, E-values, and strict time-order checks. No estimator in this crate turns observational data into an intervention effect.

Inputs are never silently reordered, imputed, or de-duplicated. Invalid data and impossible graph operations return `CausalError`.
