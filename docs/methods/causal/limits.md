# Limits

The causal routines assume finite complete vectors and a caller-selected model specification. The Granger calculation is linear, uses one shared lag order, and can fail on collinearity. The graph type rejects cycles, so feedback systems need a time-unrolled representation outside this API.

None of these utilities resolve unobserved confounding, selection bias, measurement error, nonstationarity, or causal semantics from data alone.
