import json
from lawsynth_bench.dataset import load_observations, dump_observations
def test_round_trip(tmp_path, rows):
    path = tmp_path / "results.json"; path.write_text(json.dumps(dump_observations(rows)))
    assert load_observations(path) == rows
