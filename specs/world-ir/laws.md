# Laws and dependency graph

A law target names a state variable. Its expression is an ordered expression tree; addition and multiplication are not reordered by World IR. `World::dependency_graph` and `DiscreteWorld::dependency_graph` return a lexically ordered map from each target to the lexically ordered set of symbols read by that expression. This graph records syntactic reads, including self-dependencies; it is not a causal-identification result.

Construction validates references by default before a world is returned. It does not prove numerical existence, stability, positivity, acyclicity, or suitability for a selected solver. Runtime evaluation separately rejects division by zero, invalid logarithms, unknown environment values, and non-finite results.
