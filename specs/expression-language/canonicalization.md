# Canonicalization and simplification

`to_canonical_string` is a structural representation, not source syntax: constants render as `constant:{value:.17e}`, symbols as `symbol:<id>`, and composite nodes as tagged `unary:<Debug-op>(...)` or `binary:<Debug-op>(left,right)`. It preserves tree order and distinguishes associativity. Its FNV-1a fingerprint is stable but non-cryptographic.

`print` is the parseable interchange rendering. Constants use 17-digit scientific notation; negation prints `-(...)`; named functions print `name(...)`; every binary operation prints `(left<op>right)`. The printer preserves AST structure and is unambiguous.

`simplify` performs deterministic local recursion and reductions only: finite constant folding, double-negative elimination, `+0`, `-0`, `*0`, `*1`, `/1`, and exponent-zero/one rules. It neither commutes operands nor reassociates trees. It deliberately avoids folding division by zero, non-positive constant logarithms, and non-finite results. Simplification therefore is not a complete algebra system and should not be used as an equivalence proof.
