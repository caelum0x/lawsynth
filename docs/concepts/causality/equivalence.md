# Graph equivalence

`equivalence_class` returns a deterministic summary of a graph’s skeleton and v-structures. It distinguishes structures that share adjacency from structures that encode an unshielded collider.

The result is a structural descriptor for a supplied DAG. It does not search a data-derived equivalence class, orient partially directed graphs, or attach statistical confidence to an edge.
