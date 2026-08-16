from lawsynth.candidate import CandidateMetrics


def test_candidate_dominance_requires_at_least_one_strict_improvement():
    assert CandidateMetrics(1.0, 2).dominates(CandidateMetrics(2.0, 2))
    assert not CandidateMetrics(1.0, 2).dominates(CandidateMetrics(1.0, 2))
