# Scientific constraints

Use column selection, feature bounds, and data preparation to encode constraints that the current engine can actually enforce. State variables supplied through `--state` must exist in the input. Polynomial degree, trigonometric/rational feature switches, sparse solver, and threshold are explicit controls.

Do not infer unimplemented constraints from a name or comment. There is no current CLI flag for positivity, conservation laws, dimensional analysis, causal graph restrictions, monotonicity, inequalities, or user-defined symbolic operators. Review the fitted equations and reject models that violate domain constraints.

For a hard safety boundary, validate the model after discovery and before simulation or deployment. Keep the validator independent from the fitting result and make failure blocking; a post-hoc chart annotation is not enforcement.
