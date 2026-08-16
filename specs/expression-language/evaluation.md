# Evaluation

Evaluation receives a lexical `BTreeMap<Identifier, f64>` environment and recursively evaluates the AST. The map is deterministic for diagnostics and traversal; it has no fallback variables, parameter defaults, or implicit time symbol. Missing values fail with `UnknownSymbol`.

The result must be finite. A non-finite input constant can be constructed at the Rust enum level but is rejected at the end of evaluation and cannot be encoded in a bundle. The evaluator returns one of `UnknownSymbol`, `DivisionByZero`, `DomainError { operation: "log", input }`, or `NonFiniteResult`; it never substitutes NaN, infinity, or a sentinel value.
