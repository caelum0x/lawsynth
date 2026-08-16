from lawsynth_bench.uncertainty_coverage import coverage
def test_coverage():
    assert coverage([1, 3], [0, 0], [2, 2]) == .5
