from lawsynth_bench.dataset import Observation
from lawsynth_bench.leaderboard import rank
def test_rank_higher_score_first():
    rows = [Observation("p", "a", "f1", .7), Observation("p", "b", "f1", .9)]
    assert rank(rows, "f1")[0].implementation == "b"
