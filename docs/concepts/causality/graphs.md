# Causal graphs

`CausalGraph` stores named variables and directed edges. `add_edge` requires both endpoints to exist, rejects self-edges and duplicates, and checks acyclicity. Query parents, children, edges, variables, or a topological order after construction.

The graph represents a declared causal hypothesis. It does not infer edges from data and does not encode latent confounders, bidirected edges, selection mechanisms, or probability distributions.
