# Causal utilities

`lawsynth-causal` provides checked graph primitives and small deterministic diagnostics for time order, lag construction, linear Granger comparison, Pearson correlation, Markov-equivalence summaries, and E-values. It does not identify causal effects from observational data by itself.

Use its outputs only with explicit domain assumptions. The crate stores assumptions for auditability; it cannot test whether causal sufficiency, faithfulness, or no unmeasured confounding holds in a data set.
