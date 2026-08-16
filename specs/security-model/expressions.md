# Expressions

`lawsynth-expr` uses a closed expression AST: finite constants, identifiers,
the unary operators negate/exp/log/sin/cos, and binary add/subtract/multiply/
divide/power. World construction validates referenced identifiers and bundle
decoding rejects unknown expression tags and non-finite stored constants.

Expressions are not arbitrary source code. There is no evaluator for user
provided native extensions, shell execution, file access, network access, or
reflection. This is not a complete denial-of-service boundary: callers must
still bound input size, expression construction, and execution time.
