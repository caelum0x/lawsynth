# `lawsynth-expr`

`Expr` is the executable equation AST. It has finite constants, identifiers, unary `-`, `exp`, `log`, `sin`, `cos`, and binary `+`, `-`, `*`, `/`, and power. Parser, printer, evaluator, literals, symbols, and operator definitions share this constrained representation.

Evaluation requires an explicit numeric environment. Arbitrary function calls, assignments, strings, boolean logic, side effects, and code execution are intentionally absent from the language and cannot enter a bundle expression.
