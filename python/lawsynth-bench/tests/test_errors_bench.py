from lawsynth_bench.errors_bench import classify, error_rate
def test_error_analysis():
    assert error_rate([True, False, False]) == 2 / 3
    assert classify(["parse: bad", "parse: worse"])["parse"] == 2
