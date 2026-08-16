# Nonnegative constraints

`nonnegative_least_squares` performs projected gradient descent on least squares: each coefficient update is clamped to zero. Its configuration requires a positive finite learning rate and a nonzero iteration count.

Only elementwise non-negativity is implemented. Equality constraints, units-aware constraints, general linear constraints, and KKT optimality certification are not available here.
