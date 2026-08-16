# Algebraic expressions

Use products, quotients, powers, and sums to describe explicit scalar right-hand sides. For example, `-rate * x` supplies a continuous decay derivative when assigned to state `x`. A discrete law uses the same expression language but treats its value as the next state.

The world constructor validates each law target and, when requested, checks its referenced symbols. It does not solve for an unknown inside an algebraic relation.

Implicit equations, constraint manifolds, and differential-algebraic systems need an external solver and are outside the current World IR.
