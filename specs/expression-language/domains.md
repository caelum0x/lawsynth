# Numerical domains

The language’s representable scalar domain is finite IEEE-754 binary64. Parsing and bundle encoding reject non-finite literals. Evaluation rejects any non-finite final result even if it arose from finite operands. Negative bases to non-integer powers, overflowing exponentials, and similar platform `f64` outcomes consequently fail as `NonFiniteResult`.

`log(x)` is defined only for `x > 0`; division requires an exactly nonzero denominator. No tolerance or epsilon is applied. The expression language does not encode intervals, uncertainty, complex numbers, symbolic assumptions, saturation, or missing values.
