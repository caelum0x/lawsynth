from lawsynth_bench.graph_recovery import score
def test_directed_edges():
    assert score([("a", "b")], [("a", "b"), ("b", "a")])["recall"] == 1.0
