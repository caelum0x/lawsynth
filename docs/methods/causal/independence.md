# Pearson dependence diagnostic

`pearson_independence` returns Pearson correlation and sample size for two finite, equally long vectors. `is_near_independent(tolerance)` is only a convenience threshold on absolute correlation.

Zero correlation is not general statistical independence, particularly for nonlinear relationships. The routine has no conditional-independence test, significance calculation, or missing-data policy.
