from lawsynth_bench.baseline import compare
from lawsynth_bench.config import BenchmarkConfig
from lawsynth_bench.dataset import Observation
def test_detects_regression(rows):
    candidate = [Observation("linear", "lawsynth", "wall_time", 6.0, "ms")]
    assert compare([rows[0]], candidate, BenchmarkConfig(regression_ratio=1.1))[0].regression
