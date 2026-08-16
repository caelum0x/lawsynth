# Scalar types and validity

World values are finite `f64` scalars. Identifiers are validated at creation and stay distinct across variables and parameters. Construction rejects unknown symbols when validation is enabled; simulation rejects missing state initials, unknown overrides, and non-finite values.

Expression evaluation fails on undefined symbols and non-finite results. Treat such errors as model or input failures, not as values to coerce to zero.

The IR has no nullable scalar type, categorical state type, tensor type, or automatic missing-data policy.
