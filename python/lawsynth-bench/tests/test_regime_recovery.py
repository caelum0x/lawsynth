from lawsynth_bench.regime_recovery import accuracy
def test_regime_accuracy():
    assert accuracy(["a", "b"], ["a", "a"]) == .5
