# Operators

Unary expressions support negation, `exp`, `log`, `sin`, and `cos`. Binary expressions support addition, subtraction, multiplication, division, and power. The parser applies arithmetic precedence and parentheses; the printer produces a readable expression for checkpoints and diagnostics.

Evaluation uses ordinary IEEE-754 scalar arithmetic and rejects a non-finite final value. A modeler must protect domains such as `log(x)` and division by zero through the data domain or law design.

There is no `abs`, `min`, `max`, comparison, Boolean, piecewise, or custom operator in the public expression grammar.
