from lawsynth_bench.metrics import mae, precision_recall_f1, rmse
def test_numeric_and_set_metrics():
    assert mae([1, 2], [2, 4]) == 1.5 and rmse([1], [2]) == 1.0
    assert precision_recall_f1(["a"], ["a"])[2] == 1.0
