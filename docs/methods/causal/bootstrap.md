# Bootstrap boundary

There is no causal bootstrap estimator in `lawsynth-causal`. The general uncertainty crate can resample scalar observations, but that procedure does not preserve time dependence, graph search uncertainty, or identification assumptions.

Do not attach bootstrap intervals to a Granger statistic and label them causal effects without an external, method-specific procedure.
