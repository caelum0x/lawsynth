from lawsynth.candidate import CandidateMetrics
from lawsynth.frontier import pareto_front


def test_pareto_front_keeps_tradeoffs_and_removes_dominated_metrics():
    assert pareto_front((CandidateMetrics(1, 1), CandidateMetrics(2, 2), CandidateMetrics(0.5, 3))) == (0, 2)
