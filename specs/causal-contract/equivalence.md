# Equivalence signature

`equivalence_class(graph)` returns a deterministic `MarkovEquivalence` signature consisting of an undirected skeleton and unshielded collider triples. Each skeleton edge is lexicographically canonicalized; triples are generated from distinct parents of the same child that have no edge in either direction.

This signature is useful for comparing DAG structure under the ordinary observed-variable Markov-equivalence criterion. It contains no orientation-completion algorithm and does not produce a CPDAG, PAG, or latent-variable equivalence class.

The function operates on a graph that already satisfies DAG invariants, so it has no error return and does not infer edges from data.
