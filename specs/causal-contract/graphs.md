# Directed acyclic graphs

`CausalGraph::new` registers unique, non-blank variable names. Adding a blank or duplicate variable returns `DuplicateVariable`. An edge may be added only between registered distinct variables; unknown endpoints return `UnknownVariable` and self-links return `SelfEdge`.

Before insertion, `add_edge(from, to)` searches the current graph from `to` to `from`. If a path already exists, insertion returns `Cycle` and the graph is unchanged. Duplicate edges are idempotent because edges are stored in a set.

`parents`, `children`, and `topological_order` are deterministic because storage uses ordered collections. The implementation represents an observed DAG only: no latent nodes, bidirected edges, edge attributes, or cyclic structural equation models are supported.
