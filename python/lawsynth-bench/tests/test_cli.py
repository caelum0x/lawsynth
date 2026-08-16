import json
from lawsynth_bench.cli import main
from lawsynth_bench.dataset import dump_observations
def test_cli_summary(tmp_path, rows, capsys):
    path = tmp_path / "rows.json"; path.write_text(json.dumps(dump_observations(rows)))
    assert main(["summarize", str(path), "--format", "json"]) == 0
    assert json.loads(capsys.readouterr().out)["observation_count"] == 2
