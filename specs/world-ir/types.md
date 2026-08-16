# Types and construction invariants

`Variable = { id: Identifier, role: VariableRole, unit: Unit? }`, where `VariableRole` is exactly `State`, `Control`, `Exogenous`, `Observed`, `Latent`, or `Derived`. A `Parameter = { id: Identifier, value: finite f64, unit: Unit? }` is constant for a run.

`ContinuousLaw = { target: Identifier, expression: Expr }` means `d target / dt = expression`. `DiscreteLaw` has the same fields and means `target[t+1] = expression[t]`; all discrete right-hand sides read the old state. A `World` stores continuous laws and a `DiscreteWorld` stores discrete laws. Both use lexical `BTreeMap` ordering by identifier for deterministic traversal.

Construction rejects duplicate variable IDs, duplicate parameter IDs, a parameter sharing a variable ID, a law targeting a missing or non-state variable, duplicate laws, and any state variable without exactly one law. The default configuration additionally rejects expression symbols that are neither a declared variable nor parameter, and applies unit validation. `WorldConfig { validate_expression_symbols, validate_units }` may disable either validation only for explicitly controlled import workflows; it does not relax structural invariants or finite parameters.

For a continuous world, a checked law must have dimension `dimension(target) / time`; for a discrete world it must have `dimension(target)`. A law is skipped only if its target has no unit. Symbols with no unit are absent from the dimension environment, so expressions referencing them fail unit inference when unit validation reaches that law.
