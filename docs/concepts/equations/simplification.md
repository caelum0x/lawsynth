# Simplification

`Expr::simplify()` performs deterministic local reductions: constant folding when finite, double-negation removal, neutral-element removal, and selected zero/one rewrites. Discovery simplifies the weighted feature sum before it records a law.

The routine preserves tree ordering. It does not reorder commutative operands, factor common terms, cancel symbolic factors, or prove identities that require domain assumptions.

Compare canonical strings when you need structural identity. Compare evaluations over a controlled domain when you need numerical agreement.
