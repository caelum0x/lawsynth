from lawsynth_bench.trajectory_accuracy import score
def test_trajectory_scores():
    assert score([1, 2], [2, 2]) == {"mae": .5, "rmse": 2 ** -.5}
