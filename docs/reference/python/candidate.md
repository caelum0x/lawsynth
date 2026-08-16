# Candidate metrics

`CandidateMetrics(mean_squared_error, complexity)` represents non-negative finite error and non-negative structural complexity. `dominates(other)` implements strict Pareto dominance: it is no worse in both measures and better in at least one.

This is a client-side comparison type. The current native discovery call returns a selected world rather than a Python frontier or a serializable candidate collection.
