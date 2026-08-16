# Linear Granger comparison

`granger_test` fits restricted and unrestricted linear autoregressions by solving normal equations. The restricted design contains an intercept and lagged effect values; the unrestricted design adds lagged cause values. It returns both SSE values, an F-style statistic, lag, and effective observations.

It validates sample size, length, finiteness, and singular designs. It does not calculate a p-value, correct for multiple testing, choose lag automatically, or convert predictive precedence into a causal claim.
