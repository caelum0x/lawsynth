from lawsynth.assumptions import DependencyAssumptions, EdgeAssumption
from lawsynth.errors import ValidationError
from lawsynth.graph import DependencyEdge


def test_assumptions_filter_forbidden_edges_and_reject_contradictions():
    forbidden = EdgeAssumption("x", "y")
    assert not DependencyAssumptions(forbidden=frozenset((forbidden,))).permits(DependencyEdge("x", "y", 1, 0.9))
    try:
        DependencyAssumptions(frozenset((forbidden,)), frozenset((forbidden,)))
    except ValidationError:
        pass
    else:
        raise AssertionError("contradictory assumptions accepted")
