# Confounding sensitivity

`e_value(risk_ratio)` accepts a finite positive observed risk ratio. For ratios below one it uses the reciprocal; it returns `RR + sqrt(RR * (RR - 1))`, with an E-value of one exactly when the ratio is one.

The returned `ConfoundingBound` preserves the originally supplied risk ratio and the calculated E-value. Non-positive, NaN, or infinite inputs return `InvalidParameter("risk_ratio")`.

This is a scalar sensitivity summary. It is not an adjustment set, a causal effect estimator, or evidence that a specific confounder exists.
