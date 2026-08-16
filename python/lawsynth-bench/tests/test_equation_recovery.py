from lawsynth_bench.equation_recovery import score
def test_equation_order_normalization():
    assert score("x + y", "y+x")["exact"] == 1.0
