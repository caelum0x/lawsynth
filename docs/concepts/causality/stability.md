# Stability of claims

`granger_test` reports the nested-regression F statistic and residual sums of squares for one configured lag order. `pearson_independence` reports a correlation diagnostic. Re-run those diagnostics across plausible preprocessing choices, sampling windows, and lag settings before relying on predictive structure.

Neither utility supplies p-values, multiple-testing correction, stationarity tests, bootstrap inference, or a causal effect estimate. Report the raw setup and diagnostics instead of reducing them to a causal label.
