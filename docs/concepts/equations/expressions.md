# Expression IR

`Expr` has `Constant`, `Symbol`, `Unary`, and `Binary` variants. A symbol uses a validated core `Identifier`; its numerical value comes from the evaluation environment. The evaluator returns an error for a missing symbol or a non-finite result.

`symbols()` returns the referenced identifiers, and `fingerprint()` derives a stable structural hash from the canonical tree representation. A fingerprint supports ordering and cache keys; it is not a cryptographic digest.

The IR has no vectors, matrices, conditionals, comparisons, user functions, random draws, or side effects.
