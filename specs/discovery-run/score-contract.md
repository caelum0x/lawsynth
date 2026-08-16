# Score contract

`CandidateMetrics` has two minimization objectives: mean squared error and
expression complexity. Sparse-branch MSE is total residual sum of squares
divided by the number of aligned derivative observations times the number of
requested states. Symbolic-branch MSE is the mean of its per-state calibrated
MSE values.

Complexity is the scalar expression AST node count: a constant or symbol costs
one; unary and binary nodes cost one plus their operands. It does not depend
on equation printer formatting.

The discovery executor Pareto-filters these two metrics; it does not apply the
optional weighted ranking helper. `weighted_rank` is separately available with
default weights 1.0 for error and 0.01 for complexity, validates finite
nonnegative weights, and ties by original input index.
