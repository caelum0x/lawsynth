# Sensitivity summary

`e_value` accepts a positive finite risk ratio, reciprocates ratios below one, and returns the standard E-value expression `RR + sqrt(RR × (RR - 1))`, with one mapped to one.

It is a scalar summary of an already supplied risk ratio. It does not estimate the risk ratio, model confounders, or validate the assumptions under which an E-value is meaningful.
