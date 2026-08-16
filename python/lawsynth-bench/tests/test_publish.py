import json
from lawsynth_bench.publish import write_report
def test_atomic_report_write(tmp_path):
    path = write_report({"valid": True}, tmp_path / "report.json")
    assert json.loads(path.read_text()) == {"valid": True}
