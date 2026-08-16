from lawsynth_bench.performance import time_observations
def test_only_time_units(rows):
    assert len(time_observations(rows)) == 2
