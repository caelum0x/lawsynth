# Equivalence graph

`EquivalenceGraph` stores members under the canonical form reached by repeatedly applying local normalization up to `max_passes`. Equivalence compares those bounded normalized canonical strings. Extraction chooses the lowest AST node count, breaking ties canonically.

This is a compact, safe rewrite facility, not a full congruence-closure e-graph or unbounded equality saturation engine. Its reported equivalence is limited to the implemented rewrite rules and pass budget.
