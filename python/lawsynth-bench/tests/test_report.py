from lawsynth_bench.report import build
def test_report_is_structured(rows):
    report = build(rows)
    assert report["observation_count"] == 2 and report["summaries"][0]["mean"] == 5.0
