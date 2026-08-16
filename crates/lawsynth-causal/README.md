# lawsynth-causal

`lawsynth-causal` provides auditable causal-structure primitives without making
causal conclusions on behalf of a caller. `CausalGraph` is a DAG with explicit
variables and acyclicity checks. `granger_test` fits nested autoregressions by
ordinary least squares and reports the resulting predictive F statistic.

Use graph and time-order validation to encode scientific assumptions, then keep
those assumptions with the result. Granger predictability is not evidence of an
interventional effect; unmeasured confounding and contemporaneous effects are
outside this crate's estimator.
