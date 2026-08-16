# Feature libraries

Sparse solvers accept a numeric `RegressionProblem`; feature construction lives in `lawsynth-features`. Polynomial, interaction, trigonometric, rational, and delayed terms must be evaluated before fitting, with row alignment preserved by the caller.

The solver cannot verify that a numeric column corresponds to a dimensionally valid symbolic term. Keep the feature metadata alongside the matrix when turning coefficients back into laws.
