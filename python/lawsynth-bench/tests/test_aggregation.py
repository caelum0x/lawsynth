from lawsynth_bench.aggregation import summarize
def test_summary(rows):
    summary = summarize(rows)[0]
    assert (summary.count, summary.mean, summary.median) == (2, 5.0, 5.0)
