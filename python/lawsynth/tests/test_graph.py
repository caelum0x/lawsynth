from lawsynth.graph import DependencyEdge, DependencyGraph


def test_dependency_graph_retains_unique_lagged_edges():
    edge = DependencyEdge("x", "y", 2, -0.5)
    assert DependencyGraph((edge,)).edges == (edge,)
