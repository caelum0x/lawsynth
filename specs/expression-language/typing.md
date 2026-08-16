# Scalar and dimensional typing

Every AST node is scalar. The expression crate does not attach a static type to it. `lawsynth-units::infer_expression_dimension` supplies the optional dimensional discipline used by World IR.

Constants are dimensionless. Symbol dimensions are looked up from a complete map. Negation preserves a dimension. Addition and subtraction require equal dimensions. Multiplication and division combine exponents. `exp`, `log`, `sin`, and `cos` require and return dimensionless values. A dimensionless-base power is dimensionless; otherwise the exponent must be an integer constant in `[-128, 127]`. Missing symbol units, incompatible dimensions, and exponent overflow are errors.

Typing is not value-domain proof: a well-dimensioned `log(x)` can still fail at runtime for a non-positive `x`.
